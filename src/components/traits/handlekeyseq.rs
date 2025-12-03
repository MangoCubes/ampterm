use std::fmt::Debug;

use crossterm::event::KeyEvent;
use serde::de::DeserializeOwned;

use crate::{
    action::action::Action,
    components::traits::renderable::Renderable,
    config::{keybindings::KeyBindings, keyparser::KeyParser},
};

#[derive(Clone)]
pub enum KeySeqResult {
    NoActionNeeded,
    ActionNeeded(Action),
}

pub trait PassKeySeq: Renderable {
    fn handle_key_seq(&mut self, keyseq: &Vec<KeyEvent>) -> Option<KeySeqResult>;
}

pub struct ComponentKeyHelp {
    pub name: String,
    pub bindings: Vec<KeyBindingHelp>,
}

pub struct KeyBindingHelp {
    pub keyseq: String,
    pub desc: String,
}

pub trait HandleKeySeq<T: PartialEq + DeserializeOwned + Debug + Clone + ToString>:
    Renderable
{
    /// Optionally, a componen may have a set of subcomponents that has keybinds. This function is
    /// called just before the key sequence is matched against this component's keybinding. If this
    /// function returns something other than None, it means that the key sequence matched against
    /// something in the subcomponent, and this component should not override that.
    fn pass_to_lower_comp(&mut self, keyseq: &Vec<KeyEvent>) -> Option<KeySeqResult> {
        let _ = keyseq;
        None
    }

    fn handle_local_action(&mut self, action: T) -> KeySeqResult;

    fn get_keybinds(&self) -> &KeyBindings<T>;

    fn handle_key_seq(&mut self, keyseq: &Vec<KeyEvent>) -> Option<KeySeqResult> {
        if let Some(res) = self.pass_to_lower_comp(keyseq) {
            Some(res)
        } else if let Some(res) = self.get_keybinds().get(keyseq) {
            Some(self.handle_local_action(res.clone()))
        } else {
            None
        }
    }

    fn get_other_helps(&self) -> Vec<KeyBindingHelp> {
        vec![]
    }
    fn get_name(&self) -> &str;

    fn get_help(&self) -> ComponentKeyHelp {
        let mut current: Vec<KeyBindingHelp> = self
            .get_keybinds()
            .iter()
            .map(|(ks, a)| KeyBindingHelp {
                keyseq: KeyParser::keyseq_to_string(ks),
                desc: a.to_string(),
            })
            .collect();
        current.extend(self.get_other_helps());
        ComponentKeyHelp {
            bindings: current,
            name: self.get_name().to_string(),
        }
    }
}
