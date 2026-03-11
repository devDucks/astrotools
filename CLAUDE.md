# CLAUDE.md — AI Assistant Guide for astrotools

## Project Overview

`astrotools` is a Rust library (v0.8.0) that provides base traits, types, and utilities for implementing the **Lightspeed protocol** — a framework for building drivers for astronomical equipment (cameras, mounts, focusers, filter wheels, power boxes). It uses MQTT as its communication layer and JSON/serde for message serialization.

## Repository Structure

```
astrotools/
├── src/
│   ├── lib.rs          # Crate root: LightspeedError, Lightspeed trait, module exports
│   ├── base.rs         # PropertyManager trait
│   ├── types.rs        # DeviceType enum, DevType trait
│   ├── properties.rs   # Property system: PropValue, Property<T>, RangeProperty<T>, ChoiceProperty<T>
│   └── filter_wheel.rs # FilterWheel trait
├── Cargo.toml          # Package manifest (edition 2021, GPL-3.0-or-later)
├── Cargo.lock          # Locked dependency versions (gitignored)
├── README.md           # Minimal project description
└── LICENSE             # GPL-3.0-or-later
```

## Core Architecture

### Key Traits

| Trait | File | Purpose |
|-------|------|---------|
| `Lightspeed` | `lib.rs` | Core device trait: `sync_state`, `update_property` |
| `PropertyManager` | `base.rs` | Low-level property access: `fetch_props`, `update_property` |
| `Prop<T>` | `properties.rs` | Property interface: `validate`, `update`, `update_int` |
| `DevType` | `types.rs` | Device type identification: `dev_type() -> DeviceType` |
| `FilterWheel` | `filter_wheel.rs` | Filter wheel operations: slot control, unidirectional mode |

### Property System

The property system is central to this crate. Three property types are provided:

- **`Property<T>`** — Basic property with a value and read/write permission
- **`RangeProperty<T>`** — Property with `min`/`max` bounds validation
- **`ChoiceProperty<T>`** — Property restricted to a set of valid choices

All properties use:
- `Permission` enum: `ReadOnly` | `ReadWrite`
- `PropValue` enum (untagged serde): `Int(u32)`, `Bool(bool)`, `Str(String)`, `Float(f32)`
- `UpdatePropertyRequest` — JSON-deserializable update payload from MQTT

### Error Handling

`LightspeedError` (in `lib.rs`) wraps errors with `serde` serialization support:
- Converts from `PropertyErrorType` and `std::io::Error`
- `PropertyErrorType` variants: `CannotUpdateReadOnlyProp`, `InvalidValue`, `InvalidChoice`, `ValueOutOfRange`

## Development Workflow

### Prerequisites

- Rust toolchain (edition 2021, tested with 1.93.1+)
- Cargo (comes with Rust)

### Essential Commands

```bash
# Build
cargo build

# Run all tests (12 unit tests)
cargo test

# Build in release mode
cargo build --release

# Check for errors without producing artifacts
cargo check

# Format code (use before committing)
cargo fmt

# Run linter
cargo clippy
```

### Testing

Tests are co-located with source code using Rust's built-in `#[cfg(test)]` modules:

- `properties.rs` — 9 unit tests + 3 serialization tests in separate `unit_tests` and `serialization_tests` modules
- `lib.rs` — 1 error serialization test

All 12 tests must pass before committing. Run `cargo test` to verify.

### Branching

- `main` — stable release branch (on remote `origin`)
- `master` — local development branch
- Feature branches follow: `claude/<description>-<id>` pattern

## Code Conventions

### Naming

- **Types, Traits, Enums:** `PascalCase` — e.g., `PropertyErrorType`, `RangeProperty`, `UpdatePropertyRequest`
- **Functions, Methods, Variables:** `snake_case` — e.g., `fetch_props`, `update_property`, `dev_type`
- **Constants:** `SCREAMING_SNAKE_CASE` (if any added)

### Design Patterns

1. **Trait-based abstraction** — Prefer traits over concrete types for device interfaces
2. **Generics for properties** — Use `Property<T>`, `RangeProperty<T>`, `ChoiceProperty<T>` with appropriate type bounds
3. **`Result<T, LightspeedError>`** — All fallible operations return this error type
4. **Validate before mutate** — Call `validate()` before applying changes in property updates
5. **`serde` on public data** — All types sent over MQTT must implement `Serialize`/`Deserialize`

### Serialization

- Use `#[derive(Serialize, Deserialize)]` from `serde` for public data types
- `PropValue` uses `#[serde(untagged)]` for flexible JSON parsing
- Error types use `serde` to produce JSON responses over MQTT

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `serde` | 1.0 | Serialization framework (with `serde_derive` feature) |
| `serde_json` | 1.0 | JSON encoding/decoding for MQTT messages |

No external test frameworks — Rust's built-in testing is used exclusively.

## Important Notes for AI Assistants

1. **This is a library crate** — there is no `main.rs` or binary. Changes should maintain API compatibility unless a breaking version bump is intended.
2. **GPL-3.0-or-later license** — any new code added must be compatible with this license.
3. **Keep dependencies minimal** — the crate intentionally has only 2 direct dependencies. Justify any additions.
4. **Run `cargo test` after any change** — all 12 tests must pass.
5. **Run `cargo clippy` and `cargo fmt`** — the codebase follows standard Rust formatting; no warnings should be introduced.
6. **Property permission is enforced at runtime** — `ReadOnly` properties must never be updatable; tests cover this.
7. **MQTT integration context** — this library is consumed by device drivers that communicate via MQTT. Properties map directly to MQTT topics/payloads.
8. **Versioning** — currently at 0.8.0 in `Cargo.toml`; bump appropriately for breaking vs non-breaking changes.
