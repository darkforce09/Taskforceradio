# Foundation

The lowest layer: [`tfar_foundation`](/crates/foundation/tfar_foundation/README.md), the geometry
in Arma's frame and the parsers for the strings Arma sends. Nothing here knows TeamSpeak, audio or a
mod version. A foundation crate depends on no other category (`cargo xtask verify crate-tiers`).
