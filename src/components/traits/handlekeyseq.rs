use crossterm::event::KeyEvent;

use crate::{action::Action, components::traits::renderable::Renderable};

#[derive(Clone)]
pub enum KeySeqResult {
    NoActionNeeded,
    ActionNeeded(Action),
}

pub trait HandleKeySeq: Renderable {
    fn handle_key_seq(&mut self, keyseq: &Vec<KeyEvent>) -> Option<KeySeqResult>;
}
