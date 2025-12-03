use std::{collections::HashMap, fmt::Debug};

use crossterm::event::KeyEvent;
use derive_deref::{Deref, DerefMut};
use serde::{de::DeserializeOwned, Deserialize, Deserializer};

use crate::{
    components::traits::handlekeyseq::KeySeqResult, config::keyparser::KeyParser, trace_dbg,
};

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct KeyBindings<T: Clone + PartialEq + DeserializeOwned + Debug>(
    pub HashMap<Vec<KeyEvent>, T>,
);

pub struct KeyBindingHelp {
    pub keyseq: String,
    pub hide: bool,
    pub desc: String,
}

impl<T: PartialEq + DeserializeOwned + Debug + Clone> Default for KeyBindings<T> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<'de, T: PartialEq + DeserializeOwned + Debug + Clone> Deserialize<'de> for KeyBindings<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let parsed_map = HashMap::<String, T>::deserialize(deserializer)?;

        let keybindings: HashMap<Vec<KeyEvent>, T> = parsed_map
            .into_iter()
            .map(|(keyseq, action)| (KeyParser::parse_key_sequence(&keyseq).unwrap(), action))
            .collect();

        Ok(KeyBindings(trace_dbg!(keybindings)))
    }
}

impl<T: PartialEq + DeserializeOwned + Debug + Clone> KeyBindings<T> {
    // pub fn to_help(&self) -> Vec<KeyBindingHelp> {
    //     self.0
    //         .iter()
    //         .map(|(keyseq, action)| KeyBindingHelp {
    //             keyseq: ,
    //             hide: todo!(),
    //             desc: todo!(),
    //         })
    //         .collect()
    // }
    pub fn find_action_str(&self, action: T) -> Option<String> {
        let msg = KeyParser::keyseq_to_string(self.find_action(action)?);
        Some(format!("<{}>", msg))
    }

    pub fn find_action(&self, action: T) -> Option<&Vec<KeyEvent>> {
        self.0
            .iter()
            .find_map(|(key, val)| if action == *val { Some(key) } else { None })
        // .iter()
        // .find_map(|(key, &val)| if val == value { Some(key) } else { None })
    }
}
