use crossterm::event::KeyEvent;

use crate::{action::Action, components::traits::renderable::Renderable};

pub trait HandleKeySeq: Renderable {
    fn handle_key_seq(&mut self, key: Vec<KeyEvent>) -> Option<Action>;
}
