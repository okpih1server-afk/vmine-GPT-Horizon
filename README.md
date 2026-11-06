# Veinminer for Horizon (Rust)

A native Rust port of the server-side connected-mining behavior of [Veinminer](https://modrinth.com/mod/veinminer) for [Horizon](https://github.com/horizon-rust/horizon), the Rust reimplementation of Minecraft.

This repository starts from the independently written MIT-licensed native port at [levzzz5154/horizon-veinminer](https://github.com/levzzz5154/horizon-veinminer), with its license and attribution retained. The supplied reference binary was Veinminer **2.13.0** for NeoForge; its bytecode and assets are not copied into this project.

## Current implementation

The native port currently supports:
- connected vein mining (including diagonal neighbours) through Horizon's normal player block-breaking path;
- ore and wood groups, explicit block/tool groups and per-group overrides;
- per-player session toggle and cooldown, optional V-key activation, per-block delay and merged item drops;
- sneak, tool, permission, search-radius and maximum-chain controls;
- durability protection, configurable exhaustion and mining-speed multiplier (0.1×–10×);
- JSON configuration and `/veinminer` commands;
- optional Veinmine-enchantment requirement (`requireEnchantment`).

The client preview/highlighting/pattern-selection addon is not yet ported. Host-player delayed mining, merged drops, optional V-key gating and the optional Veinmine enchantment data pack are available. LAN guest chains currently execute immediately. See [feature coverage](docs/compatibility.md) and [validation](docs/validation.md).

## Build and package

The source targets the exact Horizon and Horizon Mixins revisions pinned in `compatibility.json`. Build the source bundle locally with:

```sh
python3 tools/package.py
```

The output is a source-only `.hnmod` bundle under `target/`. The same command also builds `target/veinminer-enchantment.zip`, an optional Minecraft 26.3 data pack. CI publishes both artifacts. Install the enchantment ZIP in the world's `datapacks/` directory, enable it, then set `requireEnchantment: true` in the Veinminer JSON configuration. A compatible Horizon launcher with experimental native-mod support is required; this is not a NeoForge/Fabric JAR. See [the Horizon native-mod guide](https://github.com/horizon-rust/horizon/blob/main/docs/native-mods.md) and the [architecture notes](docs/architecture.md).

## License and provenance

- This Horizon implementation and its Rust code are MIT-licensed; see [LICENSE](LICENSE).
- Original mod: [MiraculixxT/Veinminer](https://github.com/MiraculixxT/Veinminer). The supplied NeoForge artifact declares AGPL-3.0. This repository does not contain its bytecode or assets.
- Native Horizon integration reference: [levzzz5154/horizon-veinminer](https://github.com/levzzz5154/horizon-veinminer), MIT-licensed. Its license notice is retained.
