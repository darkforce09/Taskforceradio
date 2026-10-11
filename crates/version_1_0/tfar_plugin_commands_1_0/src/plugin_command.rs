//! `PluginCommand`: the messages 1.0 plugins send each other through TeamSpeak.
//!
//! **Role:** the typed form of the four client-to-client messages of the 1.0 plugin.
//!
//! **Position:** a module of `tfar_plugin_commands_1_0`; encoded and decoded by
//! `command_codec`; built and matched by `tfar_plugin_1_0`.
//!
//! **Signals and state:** none; value types.
//!
//! **Invariants:** each variant is one row of the protocol table in
//! `/documentation/behaviour_specs/tfar_1_0_plugin.md` §4, read from
//! `reference/tfar_1_0/ts/src/plugin.cpp:1026-1068` and `CommandProcessor.cpp:326`, `:378`.

use tfar_arma_values::RadioSubtype;

/// The kind of radio a tangent broadcast keys, from its command name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TangentKind {
    /// `TANGENT`: a short-range radio.
    ShortRange,
    /// `TANGENT_LR`: a long-range radio; the receiver marks the sender able to use one.
    LongRange,
    /// `TANGENT_DD`: a diver radio, received as a short-range one.
    DiverDigital,
}

impl TangentKind {
    /// Every kind.
    pub const ALL: [Self; 3] = [Self::ShortRange, Self::LongRange, Self::DiverDigital];

    /// The command name: `TANGENT`, `TANGENT_LR` or `TANGENT_DD`.
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::ShortRange => "TANGENT",
            Self::LongRange => "TANGENT_LR",
            Self::DiverDigital => "TANGENT_DD",
        }
    }

    /// The kind whose command name is exactly `name`.
    pub fn from_wire_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.wire_name() == name)
    }
}

/// A TeamSpeak client id, as `REQVOL` carries it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TeamspeakClientId(pub u16);

/// A radio press or release, broadcast to the current channel: the game's tangent message with
/// the sender's radio abilities and nickname appended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TangentBroadcast {
    /// The kind of radio keyed.
    pub kind: TangentKind,
    /// A press (`PRESSED`); any other state text is a release, encoded `RELEASED`.
    pub is_pressed: bool,
    /// The frequency with the radio code appended.
    pub frequency: String,
    /// The transmission range in metres, rounded from [`TangentBroadcast::range_text`].
    pub range: i32,
    /// The range as the game wrote it; this is what is encoded, so a forwarded press carries the
    /// game's bytes.
    pub range_text: String,
    /// The radio subtype.
    pub subtype: RadioSubtype,
    /// The radio's class name, present in the 9-token form only.
    pub radio_classname: Option<String>,
    /// The sender can use a short-range radio (`1`).
    pub can_use_sw_radio: bool,
    /// The sender can use a diver radio (`1`).
    pub can_use_dd_radio: bool,
    /// The sender's nickname, the last token.
    pub nickname: String,
}

/// One client-to-client message of the 1.0 plugin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PluginCommand {
    /// `TANGENT`, `TANGENT_LR` or `TANGENT_DD`: a radio press or release, 8 or 9 tokens.
    Tangent(TangentBroadcast),
    /// `RELEASE_ALL_TANGENTS\tnick`: every radio of that player released.
    ReleaseAllTangents {
        /// The player's nickname.
        nickname: String,
    },
    /// `VOLUME\tnick\tvoiceVolume\ttrue|false`: the sender's voice range and whether they talk,
    /// sent on every change of their talking flag.
    Volume {
        /// The sender's TeamSpeak nickname.
        nickname: String,
        /// The sender's voice range in metres.
        voice_volume: i32,
        /// The sender talks now.
        is_talking: bool,
    },
    /// `REQVOL\t<clientId>`: a request that the receiver answers with `VOLUME` to that client;
    /// the original never sends it.
    RequestVolume {
        /// The client to answer.
        client_id: TeamspeakClientId,
    },
}
