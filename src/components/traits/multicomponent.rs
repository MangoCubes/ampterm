use color_eyre::Result;

use crate::{action::Action, components::traits::component::Component};

pub trait MultiComponent<const n: usize>: Component {
    /// Update the state of the component based on a received action. (REQUIRED)
    ///
    /// # Arguments
    ///
    /// * `action` - An action that may modify the state of the component.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Action>>` - An action to be processed or none.
    fn update(&mut self, action: Action) -> Result<[Action; n]>;
}
