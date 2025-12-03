use std::{collections::HashMap, fmt::Debug};

use crossterm::event::KeyEvent;
use derive_deref::{Deref, DerefMut};
use serde::{de::DeserializeOwned, Deserialize, Deserializer};

use crate::{
    components::traits::handlekeyseq::KeyBindingHelp, config::keyparser::KeyParser, trace_dbg,
};

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct KeyBindings<T: Clone + PartialEq + DeserializeOwned + Debug + ToString>(
    pub HashMap<Vec<KeyEvent>, T>,
);

impl<T: PartialEq + DeserializeOwned + Debug + Clone + ToString> Default for KeyBindings<T> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<'de, T: PartialEq + DeserializeOwned + Debug + Clone + ToString> Deserialize<'de>
    for KeyBindings<T>
{
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

impl<T: PartialEq + DeserializeOwned + Debug + Clone + ToString> KeyBindings<T> {
    pub fn to_help(&self) -> Vec<KeyBindingHelp> {
        self.iter()
            .map(|(ks, a)| KeyBindingHelp {
                keyseq: KeyParser::keyseq_to_string(ks),
                desc: a.to_string(),
            })
            .collect()
    }
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
