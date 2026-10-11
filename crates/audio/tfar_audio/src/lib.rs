//! The voice DSP both plugins share, one module per stage: [`sample_buffer`] (TeamSpeak's
//! interleaved 16-bit frames and saturating gain), [`dsp_filters`] (Butterworth and RBJ filters),
//! [`dynamics`] (the compressor), [`hrtf`] (binaural rendering over the IRCAM sphere),
//! [`spatial_audio`] (distance and cone gains, stereo pan) and [`playback_mixer`] (radio sounds and
//! the own-voice stream). No module knows which mod version calls it; logging goes only through the
//! `tracing` macros.

pub mod dsp_filters;
pub mod dynamics;
pub mod hrtf;
pub mod playback_mixer;
pub mod sample_buffer;
pub mod spatial_audio;
