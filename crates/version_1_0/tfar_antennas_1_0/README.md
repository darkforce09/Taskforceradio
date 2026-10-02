# Antennas for 1.0

The `tfar_antennas_1_0` crate: the 1.0 antenna and radio tower model used for terrain-aware
reception. Used by `tfar_plugin_1_0`, `tfar_radio_routing_1_0`.

## Contents

```text
crates/version_1_0/tfar_antennas_1_0/
├── Cargo.toml  the `tfar_antennas_1_0` package (lib): tier 2, workspace lints and dependencies
└── src/        the tower, the manager, the connection, the net id and the error type
```

## How it works

The game reports radio towers with `RadioTwrAdd` (`netId;range;[x,y,z]` records) and removes them
with `RadioTwrDel`. The plugin app turns each parsed record into an `Antenna` and hands it to
`AntennaManager::add`, which replaces a tower of the same `NetId` in place and ignores the null
id; `remove` drops towers by id.

For every sender, the routing crate asks `find_connection(sender, sender_range, receiver)`, which
follows `AntennaManager::findConnection` (`reference/tfar_1_0/ts/src/antennaManager.cpp:24-46`):
the candidates are the towers the sender reaches (strictly closer than its range) and that reach
the receiver (strictly closer than the tower's range); the loss of a candidate is
`d(receiver)/tower_range + d(sender)/sender_range`; the lowest loss below 1 wins, the first
tower on a tie.

The original returns a connection with loss 7 for "none". Here "none" is `None`. Callers that
take the minimum of a distance error and the antenna loss, as the voice pipeline does for a
radio effect's error level, map `None` to `NO_CONNECTION_LOSS` (7) with
`loss_or_no_connection`, which keeps the original's cap exactly.

`NetId::parse` reads `creator:object` within its own text: the original runs `strchr` over a
`string_view` that is not terminated, so an id without a colon borrows the colon of the text
after it (`reference/tfar_1_0/ts/src/datatypes.hpp:187-212`).

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_antennas_1_0  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

- `Antenna`: a tower (`new(id, position, range)`, `is_reached_by`, `reaches`, `connection_loss`).
- `AntennaManager`: `add`, `remove`, `clear`, `antennas`, `find_connection`.
- `AntennaConnection` (`antenna`, `loss`), `NO_CONNECTION_LOSS`, `loss_or_no_connection`.
- `NetId`: `new`, `parse`, `creator`, `object`, `is_null`, `NULL`.
- `Error`, `Result`, `CRATE_ROLE`, `prelude`.

## Boundaries

- Depends on: `tfar_geometry` (positions and distances); `tfar_arma_values` (the integer reader
  of `NetId::parse`); `thiserror` for the error type.
- Used by: `tfar_plugin_1_0`, `tfar_radio_routing_1_0`.
- Rules: a connection is only found through towers both ends reach, with a loss below 1
  (`the_lowest_loss_tower_below_one_is_chosen`, `a_tower_the_sender_cannot_reach_is_no_connection`);
  the net id is read within its own text (`net_id_parse_never_reads_past_its_own_text`). Tier 2 in
  `crates/version_1_0`: every workspace dependency has a lower tier and follows the edge matrix in
  `crates/README.md` (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports, §3.8 for the antenna rule.
