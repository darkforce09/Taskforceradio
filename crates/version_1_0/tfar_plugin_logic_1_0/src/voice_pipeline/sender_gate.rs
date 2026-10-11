//! The gates in front of the 1.0 voice chain: plugin state, spectators and serious mode.
//!
//! **Role:** decide, before any processing, whether a sender's frame passes untouched, is
//! silenced, or goes through the chain, as steps 1 to 4 of `processVoiceData`
//! (`reference/tfar_1_0/ts/src/plugin.cpp:467-527`); and the spectator view the chain needs.
//!
//! **Position:** a module of `tfar_plugin_logic_1_0::voice_pipeline`; called by the voice processor first,
//! and by the DSP needs for the spectator view.
//!
//! **Signals and state:** none of its own; it reads the session and refreshes the sender's
//! plugin-enabled cache through `SessionState::plugin_enabled_for`.
//!
//! **Invariants:** a sender the session does not know passes through; the local player without
//! TFAR is silence, never heard raw.

use crate::client_state::prelude::{ClientData, SessionState, TeamspeakClientId};
use crate::plugin_settings::PluginSettings;

use crate::voice_pipeline::voice_context::VoiceFrameContext;

/// How the spectator rules see a sender (`plugin.cpp:511-515`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SpectatorView {
    /// Both the local player and the sender spectate: they hear each other plainly.
    pub(crate) is_pure_spectator: bool,
    /// The local player spectates and the settings keep this sender from them
    /// (`isNotHearableInNonPureSpectator`).
    pub(crate) is_not_hearable: bool,
}

impl SpectatorView {
    /// The view of `sender` for `listener` under `settings`.
    pub(crate) fn of(
        listener: &ClientData,
        sender: &ClientData,
        settings: &PluginSettings,
    ) -> Self {
        let is_not_hearable = listener.is_spectating
            && if sender.is_enemy_to_player {
                settings.spectator_not_hear_enemies()
            } else {
                !settings.spectator_can_hear_friendlies()
            };
        Self {
            is_pure_spectator: listener.is_spectating && sender.is_spectating,
            is_not_hearable,
        }
    }

    /// Whether the spectator rules let the sender be heard at all (`isHearableInSpectator`).
    pub(crate) const fn is_hearable(self) -> bool {
        self.is_pure_spectator || !self.is_not_hearable
    }
}

/// The gate's decision for one frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GateVerdict {
    /// Leave the frame as TeamSpeak delivered it.
    PassThrough,
    /// Silence the frame.
    Silence,
    /// Run the chain with this spectator view.
    Process(SpectatorView),
}

/// Steps 1 to 4 of `processVoiceData` for `sender` (the game is known to run):
///
/// 1. an unknown sender passes through; the sender's plugin state comes from the 10 s cache or
///    `context.sender_metadata` (`isPluginEnabledForUser`);
/// 2. without the local player's data or the sender's plugin, the local player's own voice is
///    silence and anyone else passes through;
/// 3. `muteSpectators` silences one spectator for another;
/// 4. a sender the spectator rules keep away is silence in serious mode when the local player
///    is dead or the sender is not alive.
///
/// The original's serious-mode silence inside step 2 (`plugin.cpp:486-489`) needs the local
/// player alive, which needs their data, which that branch lacks; it never applies and is left
/// out.
pub(crate) fn judge_sender(
    session: &mut SessionState,
    context: &VoiceFrameContext<'_>,
    sender: TeamspeakClientId,
) -> GateVerdict {
    if session.clients.by_id(sender).is_none() {
        return GateVerdict::PassThrough;
    }
    let has_plugin_enabled =
        session.plugin_enabled_for(sender, context.sender_metadata, context.now);
    let is_own_voice = session.clients.own_client_id() == Some(sender);
    let (Some(listener), Some(sender_data)) =
        (session.clients.own(), session.clients.by_id(sender))
    else {
        return GateVerdict::PassThrough;
    };
    if !has_plugin_enabled {
        return if is_own_voice {
            GateVerdict::Silence
        } else {
            GateVerdict::PassThrough
        };
    }
    let settings = context.settings;
    if settings.mute_spectators() && listener.is_spectating && sender_data.is_spectating {
        return GateVerdict::Silence;
    }
    let view = SpectatorView::of(listener, sender_data, settings);
    let is_listener_alive = session.game.is_alive;
    let is_sender_alive = sender_data.is_alive(context.now, session.game.current_data_frame);
    if !view.is_hearable() && context.is_serious_mode && (!is_listener_alive || !is_sender_alive) {
        return GateVerdict::Silence;
    }
    GateVerdict::Process(view)
}
