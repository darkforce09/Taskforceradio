**Status:** live

# README template: domain or subsystem

**When to use:** a folder with child folders of its own that is none of the more specific kinds: a
crate's `src/` root, or a module folder that groups module folders (the effects of a radio effects crate,
the command groups of xtask's source). The
[README standard](/documentation/standards/readme_standard.md) defines every rule this template follows;
the domain kind adds Public surface.

## Skeleton

Copy the block and replace every `<…>` placeholder; each one says what goes there.

````markdown
# <Name of the domain or subsystem, in plain words: no path, no backticks>

<One to three sentences: what the domain or subsystem is responsible for.>

## Contents

```text
<repository path of the folder>/
├── <child folder>/  <what it is for: a lowercase phrase, no closing period>
├── <file>           <what it is for; entries run in case-insensitive name order>
└── tests/           <what the unit tests cover>
```

## How it works

<How a call, a frame or a message moves through the children, the main types, the threads, and the
invariants that span them. Name each child's part in one clause; the child's own README holds the
detail.>

## Public surface

- <module::item>: <what it is, and who outside the folder uses it>

## Boundaries

- Depends on: <the modules and crates the folder uses, read from its imports>
- Used by: <every user outside the folder, found with git grep>
- Rules: <the invariants a change here must keep, and the test or gate that checks each>

## Related documentation

- [<document title>](/documentation/<path to the document>) — <what it covers>
````

## Worked sample

Written for the `src/` root of `tfar_radio_routing_1_0`, as it reads once its code is written.

````markdown
# Radio routing for 1.0

Decides, for one speaking client and one listener, whether the listener hears them and over what: the
intercom, a handheld or backpack radio, a speaker radio, an antenna relay, or direct speech only.

## Contents

```text
crates/version_1_0/tfar_radio_routing_1_0/src/
├── error.rs        the routing error type
├── lib.rs          the crate header, the module tree and the re-exports
├── local_radio/    reception on the listener's own SW, LR and DD radios, with half duplex
├── prelude.rs      the route types the voice pipeline imports
├── route.rs        `Route`, the list of ways a listener hears a sender, and its ordering
├── speakers/       reception through speaker radios on the sender's frequency
└── tests/          unit tests for each route and their order
```

## How it works

`routes_for(sender, listener, game)` checks the intercom first, then the listener's own radios, then
speaker radios, and returns every route found. A route out of direct range falls back to the best
[antenna](/documentation/glossary.md#antenna-or-radio-tower) connection from `tfar_antennas_1_0`.
Distances are effective distances, with [terrain interception](/documentation/glossary.md#terrain-interception)
added. The functions are pure: they read a snapshot of the client state and write nothing.

## Public surface

- `routes_for`, `Route`, `RouteKind`: used by `tfar_voice_pipeline_1_0` once per sender per frame.

## Boundaries

- Depends on: `tfar_client_state_1_0`, `tfar_antennas_1_0`, `tfar_plugin_settings_1_0`, `tfar_geometry`.
- Used by: `tfar_voice_pipeline_1_0`, `tfar_plugin_1_0`.
- Rules:
  - the subtype is matched by its whole name (`a_subtype_matches_by_name_not_length`);
  - the intercom is checked before any radio (`intercom_wins_over_radio`).

## Related documentation

- [Audio pipeline](/documentation/architecture/audio_pipeline.md) — how routes become effects.
````
