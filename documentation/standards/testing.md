# Testing

What is tested, where tests live, and what never gets a test.

## What is tested

Core logic only:

- the game link wire format (frames, tokens, reply strings) and the game and plugin command parsers;
- the audio math: filters, dynamics, HRTF, spatial curves, mixing, compared against the golden vectors
  from `tools/tfar_golden_vectors` within a stated tolerance;
- radio routing, antennas, client state and the voice pipelines of both mods;
- the FFI layouts (`size_of`, `offset_of`) against the TeamSpeak SDK headers, and the host exports run
  against a built plugin through `tools/tfar_ts3_mock`;
- every fixed original bug in [bug fixes](/documentation/architecture/bug_fixes.md);
- the xtask gates, each with a fixture tree that holds and a perturbed copy that fails.

Never pin prose, constants, file layout or `Debug`/`Display` output; an assert that restates a
constant is not a test. No property or failpoint suites.

## Where tests live

- Unit tests sit in sibling files: `#[cfg(test)] #[path = "tests/<module>.rs"] mod tests;`; no inline
  test module (`cargo xtask verify test-placement`).
- Integration tests live in the crate's `tests/` folder.
- Test names are sentences of what holds (`a_short_pos_message_is_an_error`).
- A failing test is fixed or its code is fixed; tests are never ignored or loosened to get green.

## Running

```bash
cargo test -p <crate>          # the crates you touched, before committing
cargo test --workspace         # everything; part of cargo xtask ci ci-local
```
