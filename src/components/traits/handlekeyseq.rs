use std::fmt::Debug;

use crossterm::event::KeyEvent;
use serde::de::DeserializeOwned;

use crate::{
    action::action::Action, components::traits::renderable::Renderable,
    config::keybindings::KeyBindings,
};

#[derive(Clone)]
pub enum KeySeqResult {
    NoActionNeeded,
    ActionNeeded(Action),
}

pub trait PassKeySeq: Renderable {
    fn handle_key_seq(&mut self, keyseq: &Vec<KeyEvent>) -> Option<KeySeqResult>;
}

pub trait HandleKeySeq<T: PartialEq + DeserializeOwned + Debug + Clone>: Renderable {
    fn handle_local_action(&mut self, action: T) -> KeySeqResult;
    fn get_keybinds(&self) -> &KeyBindings<T>;
    fn handle_key_seq(&mut self, keyseq: &Vec<KeyEvent>) -> Option<KeySeqResult> {
        if let Some(res) = Self::find_action(keyseq, self.get_keybinds()) {
            Some(self.handle_local_action(res))
        } else {
            None
        }
    }
    fn find_action(keyseq: &Vec<KeyEvent>, from: &KeyBindings<T>) -> Option<T> {
        match from.get(keyseq) {
            // Test global map
            Some(a) => Some(a.clone()),
            None => match from.get(&vec![keyseq
                .last()
                .expect("Key press was detected but key sequence is empty.")
                .clone()])
            {
                // Test global map single key
                Some(a) => Some(a.clone()),
                None => None,
            },
        }
    }
}
