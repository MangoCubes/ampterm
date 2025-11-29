use crossterm::event::KeyEvent;

use crate::{action::Action, app::Mode, components::traits::renderable::Renderable};

pub enum KeySeqResult {
    NoMatch,
    NoActionNeeded,
    ActionNeeded(Action),
}

pub trait HandleKeySeq: Renderable {
    fn handle_key_seq(&mut self, mode: &Mode, keyseq: &Vec<KeyEvent>) -> KeySeqResult;
}
