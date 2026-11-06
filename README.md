# Veinminer for Horizon (Rust)

A native Rust port of the server-side connected-mining behavior of [Veinminer](https://modrinth.com/mod/veinminer) for [Horizon](https://github.com/horizon-rust/horizon), the Rust reimplementation of Minecraft.

This repository starts from the independently written MIT-licensed native port at [levzzz5154/horizon-veinminer](https://github.com/levzzz5154/horizon-veinminer), with its license and attribution retained. The supplied reference binary was Veinminer **2.13.0** for NeoForge; its bytecode and assets are not copied into this project.

## Current implementation

The native port currently supports:
- connected vein mining (including diagonal neighbours) through Horizon's normal player block-breaking path;
- ore and wood groups, explicit block/tool groups and per-group overrides;
- per-player session toggle and cooldown;
- sneak, tool, permission, search-radius and maximum-chain controls;
- durability protection, configurable exhaustion and mining-speed multiplier (0.1×–10×);
- JSON configuration and `/veinminer` commands.

It does **not** claim feature parity with the NeoForge JAR yet. Delayed per-block mining, drop merging, client-side keybind/highlighting/pattern selection, and the Veinmine enchantment addon are not implemented in this native port. See [feature coverage](docs/compatibility.md) and [validation](docs/validation.md).

## Build and package

The source targets the exact Horizon and Horizon Mixins revisions pinned in `compatibility.json`. Build the source bundle locally with:

```sh
python3 tools/package.py
```

The output is a source-only `.hnmod` bundle under `target/`. CI also publishes this file as the `veinminer-hnmod` workflow artifact after all engine and native Horizon checks pass. A compatible Horizon launcher with experimental native-mod support is required; this is not a NeoForge/Fabric JAR. See [the Horizon native-mod guide](https://github.com/horizon-rust/horizon/blob/main/docs/native-mods.md) and the [architecture notes](docs/architecture.md).

## License and provenance

- This Horizon implementation and its Rust code are MIT-licensed; see [LICENSE](LICENSE).
- Original mod: [MiraculixxT/Veinminer](https://github.com/MiraculixxT/Veinminer). The supplied NeoForge artifact declares AGPL-3.0. This repository does not contain its bytecode or assets.
- Native Horizon integration reference: [levzzz5154/horizon-veinminer](https://github.com/levzzz5154/horizon-veinminer), MIT-licensed. Its license notice is retained.
