# Audio

[`tfar_audio`](/crates/audio/tfar_audio/README.md): the voice DSP both plugin versions share
(sample frames, filters, compressor, HRTF, distance and cone gains, radio sound mixer). It depends
only on foundation; the heavy DSP crates (`sci-rs`, `biquad`, `hrtf`, `hound`) stay here
(`cargo xtask verify crate-tiers`, `dependency-firewalls`).
