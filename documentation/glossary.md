**Status:** live

# Glossary

The terms these documents use, one heading each, in alphabetical order. Documents link a term's first
use here. Original C++ names are quoted where a term maps onto one.

### Antenna or radio tower

A relay point in the 1.0 mod, placed by a mission and announced to the plugin with `RadioTwrAdd` and
`RadioTwrDel`. A transmission out of direct range can still be heard when an antenna lies within range
of both sender and listener; the connection with the least signal loss is used.

### Data frame

The game's tick of state updates: the 1.0 mod sends `DFRAME` once per round of position messages and the
plugin counts it in `currentDataFrame`. Each client's data records the frame it arrived in, and a killed
client's frame is set invalid.

### DD

The diver radio: an underwater radio for divers. Below the surface the plugin uses DD instead of an SW
radio, with its own underwater effect.

### DF2T

Direct Form II Transposed, the arrangement of a biquad filter's state that is numerically best behaved
in floating point. Every filter in the port runs in f64 DF2T.

### Dormant plugin

A plugin whose game link has never connected in this TeamSpeak session. It leaves the metadata,
nickname, mutes and channel alone and passes audio through untouched, so the 0.9 and 1.0 plugins can be
loaded at the same time.

### Game link

The connection between an Arma extension and its TeamSpeak plugin: length-prefixed frames over TCP on
127.0.0.1. It replaces the 0.9.12 named pipe and the 1.0 shared memory, which a native Linux plugin
cannot reach under Proton.

### Golden vector

The output of an original C or C++ routine (DSPFilters, SimpleComp, clunk, F3DAudio) on a fixed input,
stored as a test fixture by `tfar_golden_vectors`. The Rust audio crates are tested against it within a
stated tolerance.

### HRIR

Head-related impulse response: a measured pair of impulse responses, one per ear, for sound from one
direction. A set of them over a sphere around the head is what the HRTF convolves a voice with.

### HRTF

Head-related transfer function: the filtering by head and ears that lets a listener hear where a sound
comes from. The port applies it through the `hrtf` crate with the IRCAM HRIR sphere.

### Intercom

Vehicle crew communication in the 1.0 mod: players in the same vehicle and the same intercom slot hear
each other without a radio, with their own gain and ducking under long-range traffic.

### LR

Long-range radio: a backpack, vehicle or aircraft radio with a range of kilometres. Its key is the LR
tangent, and its messages use `TANGENT_LR`.

### Metadata block

The text between `<TFAR>` and `</TFAR>` that each plugin keeps in its TeamSpeak client metadata, saying
whether it is connected to the game, whether it is playing, and its versions. Other clients' plugins read
it to decide whether to process that client's voice, and TeamSpeak's info panel shows it.

### NEEDCFG

The answer the 1.0 extension gives to `DFRAME~` when the plugin asks for its settings again. The mod then
sends every plugin setting with `SETCFG`.

### Object interception

A 1.0 position field: how many objects, such as walls, lie between a speaker and the listener. Direct
speech behind objects is muffled by a low-pass filter whose cutoff drops with each object.

### Plugin command

A message one client's plugin sends to the other clients' plugins in its channel through TeamSpeak's
`sendPluginCommand`, such as a tangent press or `RELEASE_ALL_TANGENTS`. Only plugins named
`task_force_radio` act on it.

### PTT

Push-to-talk: speaking only while a key is held. TFAR's radio keys are tangents, and while one is held the
plugin switches TeamSpeak's voice input on and VAD off.

### RBJ

Robert Bristow-Johnson's biquad formulas from his audio EQ cookbook, used for the high-pass and low-pass
sections of the radio effects. The port takes them from the `biquad` crate.

### Serious mode

The mode in which the mission names a TeamSpeak channel for the game: the plugin moves the player into
it and mutes everyone the player should not hear in game, the dead included. Outside it, nobody is muted.

### Speaker radio

A radio with its loudspeaker turned on, so players near it hear what it receives. The mod sends the list
with `SPEAKERS`: each radio's frequencies, owner, position, volume and vehicle.

### Stereo mode

A per-frequency setting the mod sends with each radio: both ears, the left ear only or the right ear
only. Several radios can be told apart by ear this way.

### Subtype

The kind of radio sent with a tangent: `digital` (SW), `digital_lr` (LR), `dd`, `airborne` and `phone`.
The listener's plugin picks the radio effect by the sender's subtype.

### SW

Short-wave radio, TFAR's name for the personal handheld radios with a short range. Its key is the SW
tangent, and its messages use `TANGENT`.

### Sync and async message

The two kinds of extension call. A sync call waits for the plugin's answer, at most 1000 ms; an async
call, which ends in `~` and exists only in 1.0, is answered at once by the extension and delivered to the
plugin in order.

### Tangent

TFAR's word for a radio's transmit key. The mod sends `TANGENT`, `TANGENT_LR` or `TANGENT_DD` with
`PRESSED` or `RELEASED` and the frequency, range and subtype.

### Terrain interception

A position field: how much terrain lies between a speaker and the listener. The plugin adds it, times
the mission's terrain coefficient, to the effective distance of a radio transmission.

### VAD

Voice activity detection: TeamSpeak's preprocessor sending voice only while someone speaks. The plugin
turns it off while a tangent is held and restores it after the release.

### Vehicle isolation

How much a vehicle's hull damps sound between its inside and outside, sent with the vehicle id. The
isolation of both sides is summed and limited, and drives the loss and the muffling of speech and speaker
radios heard across vehicles.
