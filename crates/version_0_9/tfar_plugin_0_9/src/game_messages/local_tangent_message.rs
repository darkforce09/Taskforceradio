//! `TANGENT`, `TANGENT_LR`, `TANGENT_DD` from the game: the local push-to-talk.
//!
//! **Role:** the local tangent branch of `processGameCommand`
//! (`reference/tfar_0_9_12/ts/src/plugin.cpp:1812-1873`): store the pressed state, and on a
//! change play the local start or end sound of the radio's subtype and hand the press or the
//! delayed release to `PushToTalk`, which switches voice activation and the microphone and
//! broadcasts the tangent to the channel.
//!
//! **Position:** a method of `PluginCore`, called by the message dispatch on the game link
//! thread.
//!
//! **Signals and state:** one short lock of the server state; the sound and the push-to-talk
//! calls follow with the server lock released.
//!
//! **Invariants:** nothing is played or sent when the pressed state did not change; the
//! broadcast is the six-field plugin command every 0.9.12 plugin reads, the local player's
//! nickname last, even when the game sent no subtype.

use std::time::Instant;

use tfar_arma_values::RadioSubtype;
use tfar_game_commands_0_9::TangentKind as GameTangentKind;
use tfar_game_commands_0_9::replies::OK;
use tfar_plugin_commands_0_9::{PluginCommand, TangentKind as CommandTangentKind};
use tfar_ts3_host::ServerConnectionHandlerId;

use crate::plugin_core::PluginCore;
use crate::radio_sounds::TangentMoment;

#[cfg(test)]
#[path = "tests/local_tangent_message.rs"]
mod tests;

/// The plugin command family of the game's tangent family.
pub const fn command_tangent_kind(kind: GameTangentKind) -> CommandTangentKind {
    match kind {
        GameTangentKind::ShortRange => CommandTangentKind::ShortRange,
        GameTangentKind::LongRange => CommandTangentKind::LongRange,
        GameTangentKind::DiverDigital => CommandTangentKind::DiverDigital,
    }
}

/// The game's tangent family of a plugin command family.
pub const fn game_tangent_kind(kind: CommandTangentKind) -> GameTangentKind {
    match kind {
        CommandTangentKind::ShortRange => GameTangentKind::ShortRange,
        CommandTangentKind::LongRange => GameTangentKind::LongRange,
        CommandTangentKind::DiverDigital => GameTangentKind::DiverDigital,
    }
}

impl PluginCore {
    /// Applies the local push-to-talk at `now` on `server` and returns `OK`.
    #[expect(
        clippy::too_many_arguments,
        reason = "the fields of the game's TANGENT message, passed through as parsed"
    )]
    pub fn apply_local_tangent_message(
        &self,
        server: ServerConnectionHandlerId,
        kind: GameTangentKind,
        is_pressed: bool,
        frequency: String,
        range: i32,
        subtype: RadioSubtype,
        now: Instant,
    ) -> String {
        let applied = self.with_state(server, |state| {
            let is_change = state.radio.apply_local_tangent(kind, is_pressed, &subtype);
            let my_nickname = state.radio.my_nickname.clone();
            state.ensure_sender_dsp(&my_nickname, self.hrir());
            (is_change, my_nickname)
        });
        let Some((true, nickname)) = applied else {
            return OK.to_owned();
        };
        let moment = if is_pressed {
            TangentMoment::LocalStart
        } else {
            TangentMoment::LocalEnd
        };
        if let Some(sound) = self.sounds().tangent(&subtype, moment) {
            self.play_sound(server, &sound);
        }
        let command = PluginCommand::Tangent {
            kind: command_tangent_kind(kind),
            pressed: is_pressed,
            frequency,
            range,
            subtype,
            nickname,
        }
        .encode();
        if is_pressed {
            self.push_to_talk()
                .press(self.team_speak(), server, &command);
        } else {
            self.push_to_talk().release(server, command, now);
        }
        OK.to_owned()
    }
}
