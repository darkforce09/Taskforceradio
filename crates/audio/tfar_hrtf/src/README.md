# HRTF source

The source of `tfar_hrtf`: the sphere check and loader, the direction conversion, the block
adapter and the per-source renderer, with the render timing, the role constant, the error type and
the prelude.

## Contents

```text
crates/audio/tfar_hrtf/src/
├── block_adapter.rs                the double buffer from variable frames to whole render blocks
├── crate_role.rs                   `CRATE_ROLE`: what the crate is for, in one sentence
├── error.rs                        `Error`, the `thiserror` enum, and `Result`
├── hrir_library.rs                 `HrirLibrary`: the loaded, resampled and prepared sphere
├── hrtf_source.rs                  `HrtfSource`: per-source state, the voice-frame calls, `reset`
├── lib.rs                          the crate root: module header, `mod` lines and re-exports
├── listener_relative_direction.rs  `ListenerRelativeDirection`: Arma axes to the sphere's axes
├── prelude.rs                      the main types and the latency, for glob import
├── render_timing.rs                the sample rate, block sizes and the added latency
├── sphere_file_check.rs            the structural check of a sphere file before `hrtf` parses it
└── tests/                          one unit test file per module, plus the shared sphere loader
```

## How it works

`hrir_library.rs` reads the file, passes it through `sphere_file_check.rs` (which refuses every
input that would make the `hrtf` crate panic), lets the crate resample and transform it, and keeps
one prepared `hrtf::HrtfProcessor`. `hrtf_source.rs` copies that processor per source and pairs it
with `block_adapter.rs`: each input sample goes into the pending block at the position the output
sample comes out of the rendered block, and a full pending block is rendered by the source's
renderer (direction and gain from the latest call, gliding from the previous block's). The numbers
the three agree on live in `render_timing.rs`. `listener_relative_direction.rs` holds the only
axis convention: Arma (x east, y north, z up, compass heading) to the listener's (right, forward,
up) to the sphere's (x right, y up, z backwards). Each module attaches its sibling test file in
`tests/` with `#[path]`; `lib.rs` attaches `tests/real_sphere.rs` under `cfg(test)`.

## Public surface

- `HrirLibrary`, `HrtfSource`, `ListenerRelativeDirection`, `SAMPLE_RATE_HZ`,
  `CONVOLUTION_BLOCK_FRAMES`, `INTERPOLATION_STEPS`, `RENDER_BLOCK_FRAMES`, `ADDED_LATENCY_FRAMES`,
  `CRATE_ROLE`, `Error`, `Result` and `prelude`, re-exported at the crate root.

## Boundaries

- Depends on: `hrtf`, `glam`, `thiserror`, `tfar_geometry`, `tfar_sample_buffer`.
- Used by: `tfar_client_state_0_9`, `tfar_client_state_1_0`, `tfar_voice_pipeline_0_9`,
  `tfar_voice_pipeline_1_0`.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list), and
  the `hrtf` processor is only called with whole render blocks; tests live in `tests/`
  (`cargo xtask verify test-placement`).
