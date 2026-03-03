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

Native `.node` binaries are built for 11 target architectures and published as separate npm packages under the `@kovapatrik/` scope. The root package acts as an orchestrator.

Publishing uses [npm trusted publishers](https://docs.npmjs.com/trusted-publishers) (OIDC). The `id-token: write` permission in `release.yml` lets GitHub issue a short-lived OIDC token that npm verifies. Trusted publisher must be configured for each `@kovapatrik/` npm package on npmjs.com pointing to `release.yml`.

Two secrets are required:
- **`RELEASE_TOKEN`** — fine-grained PAT with `Contents: Read & Write` and `Pull requests: Read & Write`. Used by `release-please` so that the GitHub Release it creates triggers `release.yml`. Releases created with `GITHUB_TOKEN` do not trigger other workflows (GitHub limitation).
- **`NPM_TOKEN`** — npm token for publishing. Passed as `NODE_AUTH_TOKEN` to the publish step.

## CI / Release Workflows

| Workflow | Trigger | Purpose |
|---|---|---|
| `ci.yml` | push to `main`, PRs | Lint + native debug build |
| `release-please.yml` | push to `main` | Opens/updates release PRs; bumps version + changelog; pushes tag on merge |
| `release.yml` | GitHub release published / manual | Builds all 11 targets, publishes to npm |

### Stable release flow

1. Merge PRs using [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `chore:`, etc.)
2. `release-please` auto-opens a release PR that bumps `package.json` and writes `CHANGELOG.md`
3. Review and merge the release PR
4. `release-please` creates a GitHub Release with a `v*.*.*` tag
5. `release.yml` triggers → builds all 11 targets → publishes to npm

### Beta release flow

1. Go to **Actions → Release → Run workflow**
2. Enter the desired dist-tag (`beta`, `alpha`, `next`, etc.)
3. Builds all 11 targets and publishes under that tag

### Conventional Commits → version bumps

| Commit prefix | Version bump |
|---|---|
| `fix:` | patch |
| `feat:` | minor |
| `feat!:` or `BREAKING CHANGE:` | major |
| `chore:`, `docs:`, `refactor:` | no release |
