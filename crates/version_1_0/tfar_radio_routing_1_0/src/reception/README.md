# Reception rules

`is_over_radio` and the three rules it combines: the vehicle intercom, a radio the listener
carries, and the loudspeaker radios.

## Contents

```text
crates/version_1_0/tfar_radio_routing_1_0/src/reception/
├── intercom.rs     the intercom rule: same vehicle, same slot, no radio tangent
├── local_radio.rs  `is_over_local_radio`: the carried-radio rule with underwater range and towers
├── over_radio.rs   `is_over_radio`: the rules in the original's order
├── speakers.rs     the loudspeaker rule: exact frequency, not the sender's, known, above water
└── tests/          table-driven tests of every rule
```

## How it works

`is_over_radio` checks the intercom, stops for a sender without range, finds the best tower with
`AntennaManager::find_connection` (not for the listener itself), adds the carried-radio reception,
stops when the sender is beyond range without a tower, and adds the speakers with the tower's loss.
Each `HeardVia` carries the tower's loss when one connects the two.

## Boundaries

- Depends on: the crate's `routing_context` and `underwater`; `tfar_client_state_1_0`,
  `tfar_antennas_1_0`, `tfar_geometry`.
- Used by: the crate root, `mute_decision` and `apply_remote_tangent`.
- Rules: the carried-radio cases route as the original
  (`every_carried_radio_case_routes_as_the_original`); speakers follow their four exclusions
  (`the_speaker_case_table`); the intercom comes first (`the_intercom_case_table`).
