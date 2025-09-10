use color_eyre::Result;

use crate::{action::Action, components::traits::component::Component};
pub trait SyncComp: Component {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        let _ = action; // to appease clippy
        Ok(None)
    }
}
