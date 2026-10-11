# Foundation crates

The lowest layer of the library crates: the geometry and the Arma value parsers that every
positional or game-facing crate shares.

## How it works

`tfar_geometry` wraps `glam` with Arma's frame and the distance rules; `tfar_arma_values` parses the
strings Arma sends and uses `tfar_geometry` for positions. Nothing here knows about TeamSpeak, audio
or a mod version.

## Getting started

Run from the repository root:

```bash
# the unit tests of every crate in this category
cargo test \
  -p tfar_arma_values \
  -p tfar_geometry
```

## Boundaries

- Depends on: external crates only (`glam`) and the foundation category itself.
- Used by: every other category, both version tracks and their four deliverables.
- Rules: a foundation crate depends on no crate outside `crates/foundation`
  (`cargo xtask verify crate-tiers` holds the edges, `cargo xtask verify dependency-firewalls` the
  external crates); each crate declares `category = "crates/foundation"` and its tier: 0 without a
  workspace dependency, else one above the highest.

## Related documentation

