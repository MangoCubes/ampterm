use serde::Deserialize;

use crate::config::keybindings::KeyBindings;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct LocalKeyBinds {
    home: KeyBindings,
}
