# esphomeapi

Rust implementation of the [ESPHome native API](https://esphome.io/components/api.html), with Node.js bindings.

## Crates

```
esphomeapi                    Low-level ESPHome API client
  └── esphomeapi-manager      Entity/state management layer
        └── esphomeapi-manager-node  Node.js bindings (NAPI)
```

**`esphomeapi`** — Pure Rust client for the ESPHome native API. Handles TCP connections, Noise protocol encryption, protobuf serialization, and mDNS discovery. Can be used as a standalone Rust crate.

**`esphomeapi-manager`** — Higher-level abstraction over `esphomeapi`. Manages entity lifecycle (lights, switches, sensors, etc.), state subscriptions, and event routing. Can be used as a standalone Rust crate.

**`esphomeapi-manager-node`** — NAPI bridge that exposes `esphomeapi-manager` to Node.js/TypeScript. Published to npm as [`@kovapatrik/esphomeapi-manager`](https://www.npmjs.com/package/@kovapatrik/esphomeapi-manager).

## Releases

Releases are automated via [release-please](https://github.com/googleapis/release-please). Merge PRs using [Conventional Commits](https://www.conventionalcommits.org/) and a release PR is opened automatically.

| Commit prefix | Version bump |
|---|---|
| `fix:` | patch |
| `feat:` | minor |
| `feat!:` / `BREAKING CHANGE:` | major |

Pre-releases (beta, alpha, etc.) can be published manually from the [Release](../../actions/workflows/release.yml) workflow using the `workflow_dispatch` trigger.

## Contributing

See [CLAUDE.md](./CLAUDE.md) for architecture overview, build commands, and development workflow.
