# Changelog

## [0.3.0](https://github.com/kovapatrik/esphomeapi/compare/v0.2.0...v0.3.0) (2026-02-22)


### Features

* bump napi version, napi-cli version, protoc stup ([dfeb8a0](https://github.com/kovapatrik/esphomeapi/commit/dfeb8a01fe13c388a9af5239348ce54505e3703c))
* created esphome manager with some basic node bindings ([a630749](https://github.com/kovapatrik/esphomeapi/commit/a630749995fe8613f127223ea055fc1a543348fd))
* discovery method ([df1e20f](https://github.com/kovapatrik/esphomeapi/commit/df1e20fedd408c5708313af200e1455407fb8bdb))
* **esphomeapi-manager:** builder for commands, mapped all light ([d20037b](https://github.com/kovapatrik/esphomeapi/commit/d20037bac9a4fd44d2daaf32c5b01041bb456614))
* handle state events, update napi crates ([f90a2e6](https://github.com/kovapatrik/esphomeapi/commit/f90a2e61eb702d7b25809d4595a28b391cb19810))
* handling entities, more switch function ([340ee5b](https://github.com/kovapatrik/esphomeapi/commit/340ee5b2934211e8893d82be8f4303250100686b))
* logging binding for js, refactored discovery process ([#9](https://github.com/kovapatrik/esphomeapi/issues/9)) ([2761d33](https://github.com/kovapatrik/esphomeapi/commit/2761d337491f6d4502ab033521867dc167747f4f))
* subscriptions ([97eb0be](https://github.com/kovapatrik/esphomeapi/commit/97eb0be00d843c48fa12cc1003698c58d35868c8))


### Bug Fixes

* changed all commands to pnpm ([c0ce705](https://github.com/kovapatrik/esphomeapi/commit/c0ce70535c59e2c959ff1124c4b7fefdb59f8f00))
* create dirs command, fixed path for artifacts ([efdf7f2](https://github.com/kovapatrik/esphomeapi/commit/efdf7f2dec123057c3e7f655b07b731f27977a13))
* processing messages now wait for a whole packet ([28bf381](https://github.com/kovapatrik/esphomeapi/commit/28bf381e12f3fa5daac551c33a93fb55e508173e))
* refactor how entities are handled in node crate, exampels ([ce9a54c](https://github.com/kovapatrik/esphomeapi/commit/ce9a54cd0a91cfd9c349ee50c2046190a08674df))
* use header const, cut header from parsed frame ([50a2a17](https://github.com/kovapatrik/esphomeapi/commit/50a2a171f5661d3a16c9e109c916d8497d4ddb88))
