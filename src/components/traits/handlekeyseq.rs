use crossterm::event::KeyEvent;

use crate::{action::globalaction::GlobalAction, components::traits::renderable::Renderable};

#[derive(Clone)]
pub enum KeySeqResult {
    NoActionNeeded,
    ActionNeeded(GlobalAction),
}

pub trait HandleKeySeq: Renderable {
    fn handle_key_seq(&mut self, keyseq: &Vec<KeyEvent>) -> Option<KeySeqResult>;
}
