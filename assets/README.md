# Assets

Data the deliverables load at run time: the IRCAM HRIR sphere in `hrir/`, with its source,
checksum, credit and terms in `hrir/CREDITS.md`. `tfar_audio::hrtf` loads the sphere and `cargo xtask dist`
copies both files into each version's `tfar_hrir/` folder. Assets are data, never edited by hand;
every third-party asset carries its credit beside it, and `.gitattributes` keeps `*.bin` binary.

## The HRIR sphere

- Encoding: little-endian binary, the layout `HrtfSphere::Save` of mrDIMAS's
  hrir_sphere_builder writes: the magic `HRIR`, then `u32` sample rate (44100), HRIR length (512),
  point count (187) and index count (1110); 370 triangles as `u32` index triples; then per point
  its `f32` x, y, z followed by the left and the right impulse response as `f32` samples.
- Axes: right-handed, x to the listener's right, y up, z backwards; the frontal measurement lies
  at (0, 0, −1). The elevations run from −45° to +90°, so nothing is measured far below the head.
- Loading: `tfar_audio::hrtf` checks the structure first, then the `hrtf` crate resamples every response
  to 48 kHz (558 samples) and prepares it for its overlap-save convolution.
- Why this subject: the `hrtf` crate's repository and package carry no example or test sphere
  (its tests build synthetic spheres), so the first subject of the base, 1002, is used; every
  compensated subject has the same layout, so another one can replace it with no code change.
- Adding a file: a new sphere comes with its entry in CREDITS.md (source, commit, SHA-256, terms)
  and passes the `tfar_audio::hrtf` tests, which read `IRC_1002_C.bin` by name.

### Producers and consumers

- Producers: mrDIMAS's hrir_sphere_builder, from the IRCAM LISTEN compensated WAV files.
- Consumers: `tfar_audio::hrtf` (its `HrirLibrary::load` at plugin start, and its tests through
  `CARGO_MANIFEST_DIR`). The plugins load it from `tfar_hrir/IRC_1002_C.bin` in TeamSpeak's plugins
  folder, where `cargo xtask dist` puts it (with CREDITS.md) in each version folder.

