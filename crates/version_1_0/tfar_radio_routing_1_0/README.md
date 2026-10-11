# Radio routing for 1.0

The `tfar_radio_routing_1_0` crate: whether and how a 1.0 listener hears a speaker over radio,
intercom or speaker radio, the radio effect error levels, the serious-mode mute rules and the
serious channel lookup. Used by `tfar_plugin_1_0`, `tfar_voice_pipeline_1_0`.

## How it works

Every rule reads a `RoutingContext` (the session, the settings, the antenna manager and `now`)
built under the session lock, and never changes state.

`is_over_radio(context, sender, ignore)` lists every `HeardVia` of the sender for the local player,
in the order of `isOverRadio` (`reference/tfar_1_0/ts/src/clientData.cpp:267-364`):

1. **Intercom** (checked first): intercom enabled, the sender holds no tangent, both in the same
   vehicle (not `no`) on the same intercom slot (not -1).
2. A sender with range 0 (no tangent) is heard on nothing more.
3. **Carried radio** (`is_over_local_radio`, `clientData.cpp:148-265`), unless the sender is the
   listener. The short-range tangent (or flag) with a short-range or diver radio sends as short
   range, or as a diver radio when the sender is under water without a short-range radio; else the
   long-range tangent (or flag) with a long-range radio. The frequency must be on one of the
   listener's radios. Under water the submerged part of the path is stretched to the diver range
   (`70 + 230 · (1 − waves)`); beyond range a tower found by `find_connection` may still carry it
   if the sender reaches the tower. The long-range radio is preferred when the listener can use
   it; in half-duplex mode the radio being transmitted on (by class name) hears nothing.
4. Beyond its range, with no tower, the sender is heard on nothing more.
5. **Speakers** on the exact frequency, unless the speaker is the sender's own, its position is
   unknown (on an unknown owner), or it is under water (height below 0).

`is_over_radio_into(context, sender, ignore, out)` does the same into a list the caller reuses,
which it clears first; every `HeardVia` shares its radio id and vehicle with the session (`Arc`),
so a list with room for every reception routes a frame without allocating. `is_over_radio` wraps
it.

`mute_decision` follows `setGameClientMuteStatus` (`plugin.cpp:68-94`); `MuteTracker` records who
this plugin muted, so requests go out only on a change and `unmute_all` returns the tracked set;
`killed_mute_matrix` follows `processUnitKilled` (`CommandProcessor.cpp:552-587`).
`serious_mode_channel` resolves `ID:<n>` or an exact channel name. `effect_error_from_distance`,
`capped_by_antenna` and `underwater_effect_error` give the voice pipeline the error level of a
radio effect, with no antenna read as the original's loss of 7. `apply_remote_tangent` runs
`SessionState::apply_remote_tangent` with `is_over_radio` as its routing.

`reception/` turns a `RoutingContext` and a sender into `HeardVia` values; `mute/` decides mutes
on top of it; `effect_error.rs` and `underwater.rs` hold the formulas the voice pipeline shares
with the rules. `remote_tangent.rs` wires `is_over_radio` into the client state's tangent
handling, which cannot depend on this crate.

## Modules

### `src/mute/`

The plugin app asks `mute_decision` after a client's position, volume or tangent and
`killed_mute_matrix` after a kill, feeds the answers to its `MuteTracker`, and sends TeamSpeak only
the changes the tracker reports. On game end or disconnect `unmute_all` returns every client the
plugin muted; a client that leaves is forgotten with `client_left`.

### `src/reception/`

`is_over_radio` checks the intercom, stops for a sender without range, finds the best tower with
`AntennaManager::find_connection` (not for the listener itself), adds the carried-radio reception,
stops when the sender is beyond range without a tower, and adds the speakers with the tower's loss.
Each `HeardVia` carries the tower's loss when one connects the two, and shares its radio id and
vehicle with the session; `is_over_radio_into` fills a reused list, so routing a frame allocates
nothing once the list holds every reception.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_radio_routing_1_0  # the unit tests
```

## Configuration

Read from `tfar_plugin_settings_1_0::PluginSettings` through the context:

- `intercomEnabled` (default true): the intercom rule.
- `full_duplex` (default true): off, the radio being transmitted on hears nothing.
- `disableAutomaticMute` (default false): on, `mute_decision` and `killed_mute_matrix` leave
  serious-mode mutes alone.

## Public surface

- `RoutingContext`; `is_over_radio`, `is_over_radio_into`, `is_over_local_radio`;
  `apply_remote_tangent`.
- `mute_decision`, `MuteDecision`, `VOICE_RANGE_MARGIN`; `MuteTracker`; `killed_mute_matrix`,
  `MuteChanges`.
- `serious_mode_channel`, `ChannelId`, `CHANNEL_ID_PREFIX`.
- `effect_error_from_distance`, `capped_by_antenna`, `underwater_effect_error`;
  `diver_radio_distance`, `underwater_weighted_distance`; `prelude` adds `distance_underwater`.
- `Error`, `Result`.

## Boundaries

- Depends on: `tfar_client_state_1_0` (the session, the clients and `HeardVia`);
  `tfar_antennas_1_0` (the towers); `tfar_plugin_settings_1_0` (the settings above);
  `tfar_geometry` (distances, `distance_underwater`); `tfar_arma_values` (the `ID:<n>` integer);
  `thiserror`.
- Used by: `tfar_plugin_1_0`, `tfar_voice_pipeline_1_0`.
- Rules: routing reads client state and never changes it; the listener never hears itself on its
  own radio (`the_listener_never_hears_itself_on_its_own_radio`) and is never muted
  (`the_local_player_is_never_muted`). Tier 4 in `crates/version_1_0`: every workspace dependency
  has a lower tier and follows the category edge matrix
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports, §3.8 for the routing and §8 for the mute rules.
