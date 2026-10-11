//! The separator characters the SQF side puts between the parts of a message.
//!
//! These are wire format: the mission scripts of both mods build their `callExtension` strings
//! with exactly these characters.

/// Between the fields of a command: `POS\t…\t…`.
pub const FIELD: char = '\t';

/// Between the speaker records of a speaker list.
pub const SPEAKER_RECORD: char = '\x0B';

/// Between the parts of one speaker record.
pub const SPEAKER_PART: char = '\n';

/// Between the parts of one frequency entry: frequency, volume, stereo mode and, in 1.0, the radio
/// class name.
pub const FREQUENCY_PART: char = '|';

/// Between the parts of a vehicle descriptor: vehicle id, isolation and intercom slot.
pub const VEHICLE_PART: char = '\x10';

/// The last character of a command the extension queues asynchronously.
pub const ASYNC_SUFFIX: char = '~';
