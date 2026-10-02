//! `TANGENT`, `TANGENT_LR` and `TANGENT_DD` from the game: the local player's push-to-talk.

use tfar_game_commands_0_9::TangentKind;
use tfar_game_commands_0_9::prelude::RadioSubtype;

use crate::server_radio_data::ServerRadioData;

#[cfg(test)]
#[path = "tests/local_tangent.rs"]
mod tests;

impl ServerRadioData {
    /// Applies the local player's push-to-talk of the radio family `kind` and `subtype`
    /// (`reference/tfar_0_9_12/ts/src/plugin.cpp:1812-1834`): stores the pressed state and lets
    /// the local player's entry use that family with that subtype. Returns whether the pressed
    /// state changed: only then does the plugin broadcast the tangent with
    /// [`ServerRadioData::my_nickname`], play the local sound and switch the voice input.
    pub fn apply_local_tangent(
        &mut self,
        kind: TangentKind,
        is_pressed: bool,
        subtype: &RadioSubtype,
    ) -> bool {
        let is_change = self.is_tangent_pressed != is_pressed;
        self.is_tangent_pressed = is_pressed;
        if let Some(mine) = self.my_client_mut() {
            mine.allow_radio(kind);
            mine.subtype.clone_from(subtype);
        }
        is_change
    }
}
