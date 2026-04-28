use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use std::thread;
use std::time::{Duration, Instant};

use log::{error, info, warn};
use rumqttc::{Client, Event, MqttOptions, Packet, QoS};
use uuid::Uuid;

use crate::device::LightspeedDevice;
use crate::LightspeedError;

pub struct RunnerConfig {
    pub mqtt_client_id: String,
    pub broker_host: String,
    pub broker_port: u16,
    /// How often each device thread calls `tick()`. Default: 1000 ms.
    pub tick_interval_ms: u64,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            mqtt_client_id: "lightspeed".to_string(),
            broker_host: "127.0.0.1".to_string(),
            broker_port: 1883,
            tick_interval_ms: 1000,
        }
    }
}

/// Run devices under a Lightspeed-compatible MQTT broker.
///
/// Blocks until all device threads complete (i.e. until Ctrl-C is received).
/// No async runtime required — all I/O is handled via `std::thread` and
/// `rumqttc`'s synchronous client.
pub fn run<D: LightspeedDevice>(devices: Vec<D>, config: RunnerConfig) {
    let (state_tx, state_rx) = mpsc::sync_channel::<(Uuid, String)>(64);
    let tick_interval = Duration::from_millis(config.tick_interval_ms);

    // Extract dispatch closures and topic lists before devices are moved.
    let mut dispatchers: HashMap<
        Uuid,
        Box<dyn Fn(&str, &[u8]) -> Result<(), LightspeedError> + Send + Sync>,
    > = HashMap::new();
    let mut subscribe_topics: Vec<String> = Vec::new();

    for device in &devices {
        let uuid = device.id();
        dispatchers.insert(uuid, device.dispatcher());
        for suffix in device.command_topics() {
            subscribe_topics.push(format!("devices/{uuid}/{suffix}"));
        }
        info!("Registered device: {} ({})", device.name(), uuid);
    }

    // Shutdown flag shared between Ctrl-C handler and device threads.
    let shutdown = Arc::new(AtomicBool::new(false));

    // Spawn one thread per device.
    let mut handles = Vec::new();
    for mut device in devices {
        let state_tx = state_tx.clone();
        let shutdown = shutdown.clone();
        let handle = thread::spawn(move || loop {
            if shutdown.load(Ordering::Acquire) {
                device.close();
                break;
            }
            let start = Instant::now();
            device.tick(&state_tx);
            let elapsed = start.elapsed();
            if elapsed < tick_interval {
                thread::sleep(tick_interval - elapsed);
            }
        });
        handles.push(handle);
    }

    // MQTT client setup.
    let mut opts = MqttOptions::new(
        config.mqtt_client_id.clone(),
        config.broker_host.clone(),
        config.broker_port,
    );
    opts.set_keep_alive(Duration::from_secs(30));
    opts.set_max_packet_size(10 * 1024 * 1024, 10 * 1024 * 1024);
    let (client, mut connection) = Client::new(opts, 10);

    for topic in &subscribe_topics {
        if let Err(e) = client.subscribe(topic, QoS::AtLeastOnce) {
            error!("Failed to subscribe to {topic}: {e}");
        }
    }

    // State-publish thread: drains the state channel and publishes to MQTT.
    let pub_client = client.clone();
    thread::spawn(move || {
        while let Ok((uuid, json)) = state_rx.recv() {
            let topic = format!("devices/{uuid}");
            if let Err(e) = pub_client.publish(&topic, QoS::AtLeastOnce, false, json.as_bytes()) {
                error!("Publish failed for {uuid}: {e}");
            }
        }
    });

    // Ctrl-C: set shutdown flag and disconnect MQTT so the event loop exits.
    let shutdown_flag = shutdown.clone();
    let client_for_ctrlc = client.clone();
    ctrlc::set_handler(move || {
        info!("Shutdown signal received");
        shutdown_flag.store(true, Ordering::Release);
        let _ = client_for_ctrlc.disconnect();
    })
    .expect("Failed to register Ctrl-C handler");

    // Main MQTT event loop.
    for event in connection.iter() {
        match event {
            Ok(Event::Incoming(Packet::Publish(p))) => {
                let topic = p.topic.as_str();
                // Expected format: "devices/{36-char uuid}/{action}"
                if topic.len() < 46 || !topic.starts_with("devices/") {
                    warn!("Unexpected topic: {topic}");
                    continue;
                }
                let uuid_str = &topic[8..44];
                let action = &topic[45..];
                match uuid_str.parse::<Uuid>() {
                    Ok(uuid) => {
                        if let Some(dispatch) = dispatchers.get(&uuid) {
                            if let Err(e) = dispatch(action, &p.payload) {
                                error!("Dispatch error for {uuid}/{action}: {e:?}");
                            }
                        } else {
                            warn!("No device for UUID {uuid}");
                        }
                    }
                    Err(_) => warn!("Invalid UUID in topic: {topic}"),
                }
            }
            Ok(_) => {}
            Err(e) => {
                if shutdown.load(Ordering::Acquire) {
                    break;
                }
                error!("MQTT error: {e}");
            }
        }
    }

    // Wait for device threads to finish their cleanup.
    for handle in handles {
        let _ = handle.join();
    }
}
