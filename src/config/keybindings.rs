use std::{collections::HashMap, fmt::Debug};

use crossterm::event::KeyEvent;
use derive_deref::{Deref, DerefMut};
use serde::{de::DeserializeOwned, Deserialize, Deserializer};

use crate::{config::keyparser::KeyParser, trace_dbg};

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct KeyBindings<T: PartialEq + DeserializeOwned + Debug>(pub HashMap<Vec<KeyEvent>, T>);

impl<T: PartialEq + DeserializeOwned + Debug> Default for KeyBindings<T> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<'de, T: PartialEq + DeserializeOwned + Debug> Deserialize<'de> for KeyBindings<T> {
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

impl<T: PartialEq + DeserializeOwned + Debug> KeyBindings<T> {
    pub fn find_action_str(&self, action: T) -> Option<String> {
        let strs: Vec<String> = self
            .find_action(action)?
            .into_iter()
            .map(|k| KeyParser::key_event_to_string(&k))
            .collect();
        Some(format!("<{}>", strs.join("")))
    }

    pub fn find_action(&self, action: T) -> Option<Vec<KeyEvent>> {
        self.0.iter().find_map(|(key, val)| {
            if action == *val {
                Some(key.clone())
            } else {
                None
            }
        })
        // .iter()
        // .find_map(|(key, &val)| if val == value { Some(key) } else { None })
    }
}
