use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use log::{error, info, warn};
use rumqttc::{Client, Event, LastWill, MqttOptions, Packet, QoS};
use uuid::Uuid;

use crate::device::LightspeedDevice;
use crate::presence::{DeviceStatus, PresenceState, RunnerStatus};
use crate::topics;
use crate::LightspeedError;

pub struct RunnerConfig {
    pub mqtt_client_id: String,
    pub broker_host: String,
    pub broker_port: u16,
    /// Driver crate version, typically `env!("CARGO_PKG_VERSION").to_string()`.
    pub driver_version: String,
    /// How often each device thread calls `tick()`. Default: 1000 ms.
    pub tick_interval_ms: u64,
    /// MQTT keepalive. Default: 15 s. Detection latency ~= 1.5x this value.
    pub keepalive_secs: u64,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            mqtt_client_id: "lightspeed".to_string(),
            broker_host: "127.0.0.1".to_string(),
            broker_port: 1883,
            driver_version: "unknown".to_string(),
            tick_interval_ms: 1000,
            keepalive_secs: 15,
        }
    }
}

fn epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Run devices under a Lightspeed-compatible MQTT broker.
///
/// Blocks until all device threads complete (i.e. until Ctrl-C is received).
pub fn run<D: LightspeedDevice>(devices: Vec<D>, config: RunnerConfig) {
    let runner_id = Uuid::now_v7();
    let started_at = epoch_secs();
    let pid = std::process::id();

    let (state_tx, state_rx) = mpsc::sync_channel::<(Uuid, String)>(64);
    let tick_interval = Duration::from_millis(config.tick_interval_ms);

    let mut dispatchers: HashMap<
        Uuid,
        Box<dyn Fn(&str, &[u8]) -> Result<(), LightspeedError> + Send + Sync>,
    > = HashMap::new();
    let mut subscribe_topics: Vec<String> = Vec::new();
    let mut device_uuids: Vec<Uuid> = Vec::new();

    for device in &devices {
        let uuid = device.id();
        device_uuids.push(uuid);
        dispatchers.insert(uuid, device.dispatcher());
        for suffix in device.command_topics() {
            subscribe_topics.push(topics::device_cmd(uuid, suffix));
        }
        info!("Registered device: {} ({})", device.name(), uuid);
    }

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

    // MQTT client setup with LWT.
    let lwt_payload = serde_json::to_vec(&RunnerStatus {
        state: PresenceState::Offline,
        device_uuids: device_uuids.clone(),
        started_at,
        runner_version: config.driver_version.clone(),
        pid,
    })
    .expect("failed to serialize LWT payload");

    let last_will = LastWill::new(
        topics::runner_status(runner_id),
        lwt_payload,
        QoS::AtLeastOnce,
        true, // retained
    );

    let mut opts = MqttOptions::new(
        config.mqtt_client_id.clone(),
        config.broker_host.clone(),
        config.broker_port,
    );
    opts.set_keep_alive(Duration::from_secs(config.keepalive_secs));
    opts.set_max_packet_size(10 * 1024 * 1024, 10 * 1024 * 1024);
    opts.set_last_will(last_will);

    let (client, mut connection) = Client::new(opts, 10);

    // Subscribe to all device command topics.
    for topic in &subscribe_topics {
        if let Err(e) = client.subscribe(topic, QoS::AtLeastOnce) {
            error!("Failed to subscribe to {topic}: {e}");
        }
    }

    // Publish runner Online status (retained).
    let runner_online = RunnerStatus {
        state: PresenceState::Online,
        device_uuids: device_uuids.clone(),
        started_at,
        runner_version: config.driver_version.clone(),
        pid,
    };
    if let Ok(payload) = serde_json::to_vec(&runner_online) {
        if let Err(e) = client.publish(topics::runner_status(runner_id), QoS::AtLeastOnce, true, payload) {
            error!("Failed to publish runner status: {e}");
        }
    }

    // Publish per-device Online status (retained).
    for uuid in &device_uuids {
        let status = DeviceStatus {
            state: PresenceState::Online,
            runner_id,
            started_at,
            driver_version: config.driver_version.clone(),
            pid,
        };
        if let Ok(payload) = serde_json::to_vec(&status) {
            if let Err(e) = client.publish(topics::device_status(*uuid), QoS::AtLeastOnce, true, payload) {
                error!("Failed to publish device status for {uuid}: {e}");
            }
        }
    }

    // State-publish thread.
    let pub_client = client.clone();
    thread::spawn(move || {
        while let Ok((uuid, json)) = state_rx.recv() {
            let topic = topics::device_state(uuid);
            if let Err(e) = pub_client.publish(&topic, QoS::AtLeastOnce, false, json.as_bytes()) {
                error!("Publish failed for {uuid}: {e}");
            }
        }
    });

    // Ctrl-C handler: publish graceful Offline statuses, then disconnect.
    let shutdown_flag = shutdown.clone();
    let client_for_ctrlc = client.clone();
    let device_uuids_for_ctrlc = device_uuids.clone();
    let driver_version_for_ctrlc = config.driver_version.clone();
    ctrlc::set_handler(move || {
        info!("Shutdown signal received");
        shutdown_flag.store(true, Ordering::Release);

        // Best-effort graceful Offline statuses.
        for uuid in &device_uuids_for_ctrlc {
            let status = DeviceStatus {
                state: PresenceState::Offline,
                runner_id,
                started_at,
                driver_version: driver_version_for_ctrlc.clone(),
                pid,
            };
            if let Ok(payload) = serde_json::to_vec(&status) {
                let _ = client_for_ctrlc.publish(
                    topics::device_status(*uuid),
                    QoS::AtLeastOnce,
                    true,
                    payload,
                );
            }
        }
        let runner_offline = RunnerStatus {
            state: PresenceState::Offline,
            device_uuids: device_uuids_for_ctrlc.clone(),
            started_at,
            runner_version: driver_version_for_ctrlc.clone(),
            pid,
        };
        if let Ok(payload) = serde_json::to_vec(&runner_offline) {
            let _ = client_for_ctrlc.publish(
                topics::runner_status(runner_id),
                QoS::AtLeastOnce,
                true,
                payload,
            );
        }
        let _ = client_for_ctrlc.disconnect();
    })
    .expect("Failed to register Ctrl-C handler");

    // Main MQTT event loop.
    for event in connection.iter() {
        match event {
            Ok(Event::Incoming(Packet::Publish(p))) => {
                let topic = p.topic.as_str();
                match topics::parse_device_topic(topic) {
                    Some((uuid, action)) if !action.is_empty() => {
                        if let Some(dispatch) = dispatchers.get(&uuid) {
                            if let Err(e) = dispatch(action, &p.payload) {
                                error!("Dispatch error for {uuid}/{action}: {e:?}");
                            }
                        } else {
                            warn!("No device for UUID {uuid}");
                        }
                    }
                    Some(_) => {
                        // devices/{uuid} with no action — ignore (it's our own state publish loopback)
                    }
                    None => warn!("Unexpected topic: {topic}"),
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

    for handle in handles {
        let _ = handle.join();
    }
}
