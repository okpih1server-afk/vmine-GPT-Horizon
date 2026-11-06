# Feature coverage

Reference project: [MiraculixxT/Veinminer](https://github.com/MiraculixxT/Veinminer),
revision `b3bfeb39945260f857a9b79c11dc206c109a9d0c`.
Its [overview](https://github.com/MiraculixxT/Veinminer/blob/b3bfeb39945260f857a9b79c11dc206c109a9d0c/docs/overview.md)
describes the base server mod and separate client/enchantment addons. This is an
unofficial behavioral port of normal server-side vein mining, with an independent
configuration format and native Horizon adapter.

| Capability | This version |
|---|---|
| Connected veins including diagonal neighbours | Implemented; fixed-order bounded breadth-first traversal |
| Normal loot, XP, tool wear, block updates and events | Delegates every block to the existing Horizon player-mining method |
| Stone/deepslate ore and wood groups | Implemented with explicit IDs and independently chosen defaults |
| Custom blocks, tool groups and group settings | Implemented in JSON; first matching group supplies overrides |
| Correct-tool, survival, sneak and durability checks | Implemented |
| Cooldown, per-player toggles, extra hunger cost | Implemented |
| Permission restriction | Explicit name allowlist; no permissions plugin integration |
| Settings reload | Implemented with permission level 2 and error retention |
| Single player, LAN and dedicated server path | Shared native adapter type-checked; full live sessions require manual verification |
| Hold-to-mine keybind | Implemented as optional V-key hold (`keybindRequired`); press/release state is forwarded to the server, but preview/highlighting remains future work |
| Client preview/highlighting, HUD, tunnel/flat shape selector | Future work; core exposes a pure connected-block preview API |
| Veinmine enchantment addon | Implemented as an optional data pack (`veinminer-enchantment.zip`) plus `requireEnchantment` / `minEnchantLevel` server checks |
| Delayed per-block mining | Implemented for the local host: queued extra breaks run on later game ticks; LAN guest chains execute immediately because guest actions are packet-scoped |
| Mining-speed adjustment | Implemented with the upstream vein-size penalty (`miningSpeedModifier`, 0–1) plus an optional global multiplier (`miningSpeedMultiplier`, 0.1×–10×), applied by a RETURN modifier on Horizon `destroy_progress` |
| Merged drop positions | Implemented: item entities spawned during extra breaks redirect to the source block |
| Upstream `/settings`, `/groups`, `/presets`, `/blocks` editing | Future work; edit JSON and reload |
| Tags, wildcard IDs and client/server configuration packets | Future work; explicit IDs and server configuration only |
| Automatic updates | Future work |
| Native mixin bundle | Implemented: four cancellable TOML HEAD hooks, one non-cancellable HEAD hook, one RETURN modifier and one support module, applied by horizon-mixins 0.6.0 |
| `.hnmod` packaging | Implemented: deterministic source bundle, published as a CI artifact; launcher support remains experimental |

The maximum chain counts eligible selected blocks, including the original.
Search radius is Chebyshev distance around each accepted block; larger radii
can bridge gaps. This is a documented source-level contract, rather than a
promise of identical traversal ordering to upstream. Runtime changes use
Horizon's existing vanilla mining behavior and random call order for each
individual block. Mods that cancel the underlying native operation are
respected when it reports no removal. WASM `block-broken` callbacks are
after-the-fact notifications and cannot cancel mining.

No Minecraft assets, upstream mod source or artwork are distributed here.
Preparing resources, signing into Minecraft and running Horizon remain the
host application's responsibility. Generated exports preserve Horizon's own
license and third-party notices.
