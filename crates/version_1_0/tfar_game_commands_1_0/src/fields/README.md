# Game command fields

The readers of the field shapes the 1.0 game messages share: tokens by position, the original's
booleans, coordinate triples, network object ids and vehicle descriptors.

## Contents

```text
crates/version_1_0/tfar_game_commands_1_0/src/fields/
├── arma_truth.rs          `is_true`: exactly `true` or `1`, as `helpers::isTrue`
├── coordinates.rs         `[x,y]`, `[x,y,z]` or `[x,y,z,w]`, and the speaker position with its null rule
├── message_fields.rs      `MessageFields`: count checks and typed reads by index; the keep-every-field split
├── mod.rs                 the module lines and re-exports
├── network_object_id.rs   `NetworkObjectId`: `creator:object` read as `atoi` reads it
├── tests/                 unit tests of each reader against the original's parsing
└── vehicle_descriptor.rs  `VehicleDescriptor`: `no`, or name, isolation, intercom slot and velocity
```

## How it works

Every command parser reads its message through `MessageFields`: `require_count`,
`require_at_least` and `require_between` check the token count, and `text`, `nickname`, `truth`,
`number`, `integer` and `read` return a field or an error that names the command and the field.
The coordinate and vehicle readers return `None` for a malformed text; `MessageFields::read` turns
that into `Error::Malformed`. `NetworkObjectId::parse` never fails, as the original's `NetID`
never does: a text without a colon is the null id.

## Boundaries

- Depends on: `tfar_arma_values` (`parse_number`, `parse_int`, `sanitize_nickname`, the vehicle
  separator), `tfar_geometry` (`Position`), the crate's `CommandName` and `Error`.
- Used by: the crate's `commands/` parsers and `parse.rs`; `VehicleDescriptor` and
  `NetworkObjectId` are public.
- Rules: a coordinate text of any element count but 2 to 4 is rejected, never read as zero
  (`other_shapes_are_rejected_instead_of_read_as_zero`); the split keeps a trailing empty field
  (`the_split_keeps_every_field_as_helpers_split_does`).
