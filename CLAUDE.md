# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Build
```bash
pnpm build           # Release build (all workspaces)
pnpm build:debug     # Debug build
pnpm build:ts        # TypeScript compilation only
```

### Test
```bash
pnpm test            # Run Ava test suite
```

### Lint & Format
```bash
pnpm lint            # Biome linter with auto-fix
pnpm format          # Format Rust (cargo fmt) and TOML in parallel
pnpm format:rs       # Rust formatting only
```

### Rust-specific
```bash
cargo check          # Type-check without building
cargo fmt --all      # Format all Rust code
cargo clippy         # Rust linter (run in CI)
```

### Examples
```bash
pnpm example <file>  # Run a TypeScript example with dotenvx + tsx
```

## Architecture

This is a **Rust + Node.js native addon** (NAPI) monorepo for communicating with ESPHome devices. It uses `pnpm` workspaces and a Cargo workspace.

### Crate Layers

```
esphomeapi              (Layer 1 - Core protocol)
  └── esphomeapi-manager      (Layer 2 - Entity/state management)
        └── esphomeapi-manager-node  (Layer 3 - NAPI bindings for Node.js)
```

**`crates/esphomeapi`** — Low-level ESPHome API client. Handles TCP connections, Noise protocol encryption, protobuf serialization/deserialization, and mDNS discovery. Protobuf code is generated from `src/protos/` via `build.rs` using `protobuf-codegen`.

**`crates/esphomeapi-manager`** — Higher-level manager that wraps the core client. Manages entity lifecycle (lights, switches, etc.), state subscriptions, and event routing. Has a `bin/main.rs` for standalone testing.

**`crates/esphomeapi-manager-node`** — NAPI bridge compiled as `cdylib`. Exposes the manager to JavaScript/TypeScript. Uses `napi` + `napi-derive` macros. JavaScript-visible types live in `src/manager.rs` and `src/entity/`. Logger integration at `src/logger.rs` bridges Rust `tracing` to JS.

### Key Patterns

- **Async runtime:** `tokio` throughout; the NAPI layer uses `napi`'s `tokio_rt` feature.
- **Error handling:** `thiserror` for typed errors in core crates.
- **Logging:** `tracing` in Rust, bridged to a JS logger callback in the node crate.
- **Encryption:** Noise protocol (`noise-protocol` + `noise-rust-crypto`) for authenticated ESPHome connections.
- **Discovery:** `mdns-sd` for mDNS-based device discovery.

### npm Publishing

The CI matrix builds native `.node` binaries for 10 target architectures and publishes them as separate npm packages under the `@kovapatrik/` scope. The root package acts as an orchestrator. Published on git tags matching semver (releases) or semver with suffix (beta).
