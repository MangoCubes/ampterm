use crossterm::event::KeyEvent;

use crate::{action::action::Action, components::traits::renderable::Renderable};

/// If a component has this trait, this means that it may take a key input from the user.
/// Once it takes the key input, it optionally mutates its state.
/// Once complete, it optionally returns an action indicating the action that should further
/// happen.
pub trait HandleRaw: Renderable {
    fn handle_raw(&mut self, key: KeyEvent) -> Option<Action>;
}
