**Status:** live

# Template: agent brief

**When to use:** the prompt for one agent of one program slice, stored in
`documentation/program/agent_briefs/` as m<N>_<agent id>_<role>.md (m3_b2_game_link_protocol.md). Every
brief sends its agent to `shared_brief.md` first, which holds the rules all agents share (no git writes,
owned files only, compile at every save point, tests never weakened, a perturbation proof for every new
check, one command per shell call with its log, no silent deferrals) and the report format. A brief adds
only what is particular to its slice. It is live while the slice runs and a frozen record once the slice
is committed.

## Skeleton

Copy the block and replace every `<…>` placeholder; each one says what goes there. Steps are numbered
and concrete: each names the file, the behaviour with its `reference/` path and line, the library, the
test.

````markdown
**Status:** live

# M<N> · <agent id> · <role, in a few words>

You are agent <agent id>, the <role> for the TFAR Rust port. Read
`/documentation/program/agent_briefs/shared_brief.md` first. <Then read: the documents this agent's
work follows, each as a repository-root path.>

## Goal
<Two to four sentences: what exists when the agent is done, and the standard it is held to.>

## Steps
1. **<deliverable>.**
   - <what to build, with the reference file:line it ports, the library it uses, the bug-fix id it
     carries, and the test that shows it>
2. **Checks.**
   - <the commands to run, each alone, logged to target/agent-logs/<agent id>-<step>.log>

## Owned files
<Every path the agent may create or edit, as repository-root paths or globs. Everything else is read
only; needed changes elsewhere are reported as findings.>

## Checks
- <the gates and conditions the orchestrator will check before committing the slice>

## Report additions
<What this agent adds to the shared report format: a table, a count, a list of contradictions found.>

Budget: <S (150k tokens) | M (250k tokens) | L (350k tokens)>. Stop there and report what is done and
what is not.
````

## Worked sample

A shortened brief for the game link protocol crate, as one could be written for its milestone. The
sample sits in a fenced block, so no gate reads it.

````markdown
**Status:** live

# M3 · B1 · Game link protocol

You are agent B1, the protocol author for the TFAR Rust port. Read
`/documentation/program/agent_briefs/shared_brief.md` first. Then read
`/documentation/architecture/game_link_protocol.md` and `/documentation/decisions/game_link.md`.

## Goal
`tfar_game_link_protocol` encodes and decodes every frame of the game link, with the HELLO handshake,
and refuses every malformed input as an error value. It has no I/O and no threads.

## Steps
1. **Frame codec.**
   - `src/frame.rs`: `u32 LE length | u8 kind | payload`, with `postcard` payloads; a length above the
     limit is an error before any allocation.
   - Tests in `src/tests/frame.rs`: every kind round-trips; a short header, an unknown kind and an
     oversize length are each rejected.
2. **Handshake.**
   - `src/hello.rs`: magic `TFAR`, protocol number, flavour 0.9 or 1.0; a wrong magic is rejected.
3. **Checks.**
   - `cargo test -p tfar_game_link_protocol`, `cargo clippy -p tfar_game_link_protocol -- -D warnings`,
     `cargo xtask verify crate-anatomy`, each alone and logged.

## Owned files
`/crates/game_link/tfar_game_link_protocol/**`.

## Checks
- The fast stage gate passes; the README's Public surface lists every exported type.

## Report additions
The frame kind table as built: | Kind | Byte | Direction | Payload |.

Budget: S (150k tokens). Stop there and report what is done and what is not.
````
