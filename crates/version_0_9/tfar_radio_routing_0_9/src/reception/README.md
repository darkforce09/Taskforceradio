# Reception rules for 0.9

Which radios carry a sender to the local listener: the listener's own short-range, long-range and
diver radios, and the radios playing on loudspeaker.

## Contents

```text
crates/version_0_9/tfar_radio_routing_0_9/src/reception/
├── local_radio.rs       `is_over_local_radio`: the five own-radio checks in the original's order
├── mod.rs               the module lines and the re-exports
├── radio_receptions.rs  `is_over_radio(_into)` and `receptions`: own radio first, then loudspeakers
└── tests/               table-driven tests per rule
```

## How it works

`is_over_local_radio` runs the checks of `isOverLocalRadio`
(`reference/tfar_0_9_12/ts/src/plugin.cpp:989`): short range on the listener's long-range list,
short range on short range, long range on short range, long range on long range, then diver; a
later match overwrites an earlier one. Short and long range need the family held (or its ignore
flag), the frequency on the list, both sides able to use their radio and the effective distance
below the sender's range; the diver radio compares the raw distance with its reach.
`is_over_radio` adds the local reception for another client, then, while the sender is in range on
a held short- or long-range radio it can use, one reception per loudspeaker radio on the sender's
frequency that the sender does not carry, at its position or at its owner's while the owner's data
is current. `is_over_radio_into` does the same into a list the caller reuses; every reception
shares its radio id and vehicle with the server state, so it copies no text.

## Boundaries

- Depends on: `crates/version_0_9/tfar_radio_routing_0_9/src/distances.rs`,
  `crates/version_0_9/tfar_radio_routing_0_9/src/heard_via.rs`; `tfar_client_state_0_9`.
- Used by: the mute decisions and the sound planning of this crate; `tfar_voice_pipeline_0_9` and
  `tfar_plugin_0_9`.
- Rules: the check order, the strict `<` comparisons and the overwrite rule
  (`every_short_and_long_range_rule_of_the_local_reception`,
  `every_diver_radio_rule_of_the_local_reception`); the local reception first and loudspeakers in
  the game's order (`the_local_reception_comes_first_then_the_loudspeakers_in_the_games_order`);
  a warm list never reallocates (`a_thousand_frames_into_a_warm_list_never_reallocate_it`).
