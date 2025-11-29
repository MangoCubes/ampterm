use std::{collections::HashMap, fmt::Debug};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use derive_deref::{Deref, DerefMut};
use serde::{de::DeserializeOwned, Deserialize, Deserializer};

use crate::trace_dbg;

#[derive(Clone, Debug, Deref, DerefMut, Default)]
pub struct KeyBindings<T: PartialEq + DeserializeOwned + Debug>(pub HashMap<Vec<KeyEvent>, T>);

impl<'de, T: PartialEq + DeserializeOwned + Debug> Deserialize<'de> for KeyBindings<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let parsed_map = HashMap::<String, T>::deserialize(deserializer)?;

        let keybindings: HashMap<Vec<KeyEvent>, T> = parsed_map
            .into_iter()
            .map(|(keyseq, action)| (Self::parse_key_sequence(&keyseq).unwrap(), action))
            .collect();

        Ok(KeyBindings(trace_dbg!(keybindings)))
    }
}

impl<T: PartialEq + DeserializeOwned + Debug> KeyBindings<T> {
    pub fn key_event_to_string(key_event: &KeyEvent) -> String {
        let char;
        let key_code = match key_event.code {
            KeyCode::Backspace => "Backspace",
            KeyCode::Enter => "Enter",
            KeyCode::Left => "Left",
            KeyCode::Right => "Right",
            KeyCode::Up => "Up",
            KeyCode::Down => "Down",
            KeyCode::Home => "Home",
            KeyCode::End => "End",
            KeyCode::PageUp => "PageUp",
            KeyCode::PageDown => "PageDown",
            KeyCode::Tab => "Tab",
            KeyCode::BackTab => "Backtab",
            KeyCode::Delete => "Delete",
            KeyCode::Insert => "Insert",
            KeyCode::F(c) => {
                char = format!("f({c})");
                &char
            }
            KeyCode::Char(' ') => "Space",
            KeyCode::Char(c) => {
                char = c.to_string();
                &char
            }
            KeyCode::Esc => "Esc",
            KeyCode::Null => "",
            KeyCode::CapsLock => "CapsLock",
            KeyCode::Menu => "Menu",
            KeyCode::ScrollLock => "ScrLk",
            KeyCode::Media(k) => {
                char = k.to_string();
                &char
            }
            KeyCode::NumLock => "NumLk",
            KeyCode::PrintScreen => "PrtSc",
            KeyCode::Pause => "Pause",
            KeyCode::KeypadBegin => "KpBeg",
            KeyCode::Modifier(_) => "",
        };

        let mut modifiers = Vec::with_capacity(3);

        if key_event.modifiers.intersects(KeyModifiers::CONTROL) {
            modifiers.push("Ctrl");
        }

        if key_event.modifiers.intersects(KeyModifiers::SHIFT) {
            modifiers.push("Shift");
        }

        if key_event.modifiers.intersects(KeyModifiers::ALT) {
            modifiers.push("Alt");
        }

        let mut key = modifiers.join("-");

        if !key.is_empty() {
            key.push('-');
        }
        key.push_str(key_code);

        key
    }

    pub fn find_action_str(&self, action: T) -> Option<String> {
        let strs: Vec<String> = self
            .find_action(action)?
            .into_iter()
            .map(|k| Self::key_event_to_string(&k))
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
    fn parse_key_code_with_modifiers(
        raw: &str,
        mut modifiers: KeyModifiers,
    ) -> Result<KeyEvent, String> {
        let c = match raw {
            "esc" => KeyCode::Esc,
            "enter" => KeyCode::Enter,
            "left" => KeyCode::Left,
            "right" => KeyCode::Right,
            "up" => KeyCode::Up,
            "down" => KeyCode::Down,
            "home" => KeyCode::Home,
            "end" => KeyCode::End,
            "pageup" => KeyCode::PageUp,
            "pagedown" => KeyCode::PageDown,
            "backtab" => {
                modifiers.insert(KeyModifiers::SHIFT);
                KeyCode::BackTab
            }
            "backspace" => KeyCode::Backspace,
            "delete" => KeyCode::Delete,
            "insert" => KeyCode::Insert,
            "f1" => KeyCode::F(1),
            "f2" => KeyCode::F(2),
            "f3" => KeyCode::F(3),
            "f4" => KeyCode::F(4),
            "f5" => KeyCode::F(5),
            "f6" => KeyCode::F(6),
            "f7" => KeyCode::F(7),
            "f8" => KeyCode::F(8),
            "f9" => KeyCode::F(9),
            "f10" => KeyCode::F(10),
            "f11" => KeyCode::F(11),
            "f12" => KeyCode::F(12),
            "space" => KeyCode::Char(' '),
            "hyphen" => KeyCode::Char('-'),
            "minus" => KeyCode::Char('-'),
            "tab" => KeyCode::Tab,
            c if c.len() == 1 => {
                let mut c = c.chars().next().unwrap();
                if modifiers.contains(KeyModifiers::SHIFT) {
                    c = c.to_ascii_uppercase();
                }
                KeyCode::Char(c)
            }
            _ => return Err(format!("Unable to parse {raw}")),
        };
        Ok(KeyEvent::new(c, modifiers))
    }

    fn extract_modifiers(raw: &str) -> (&str, KeyModifiers) {
        let mut modifiers = KeyModifiers::empty();
        let mut current = raw;

        loop {
            match current {
                rest if rest.starts_with("ctrl-") => {
                    modifiers.insert(KeyModifiers::CONTROL);
                    current = &rest[5..];
                }
                rest if rest.starts_with("alt-") => {
                    modifiers.insert(KeyModifiers::ALT);
                    current = &rest[4..];
                }
                rest if rest.starts_with("shift-") => {
                    modifiers.insert(KeyModifiers::SHIFT);
                    current = &rest[6..];
                }
                _ => break, // break out of the loop if no known prefix is detected
            };
        }

        (current, modifiers)
    }

    pub fn parse_key_event(raw: &str) -> Result<KeyEvent, String> {
        let raw_lower = raw.to_ascii_lowercase();
        let (remaining, modifiers) = Self::extract_modifiers(&raw_lower);
        Self::parse_key_code_with_modifiers(remaining, modifiers)
    }

    pub fn parse_key_sequence(raw: &str) -> Result<Vec<KeyEvent>, String> {
        if raw.chars().filter(|c| *c == '>').count() != raw.chars().filter(|c| *c == '<').count() {
            return Err(format!("Unable to parse `{}`", raw));
        }
        let raw = if !raw.contains("><") {
            let raw = raw.strip_prefix('<').unwrap_or(raw);
            let raw = raw.strip_prefix('>').unwrap_or(raw);
            raw
        } else {
            raw
        };
        let sequences = raw
            .split("><")
            .map(|seq| {
                if let Some(s) = seq.strip_prefix('<') {
                    s
                } else if let Some(s) = seq.strip_suffix('>') {
                    s
                } else {
                    seq
                }
            })
            .collect::<Vec<_>>();

        sequences.into_iter().map(Self::parse_key_event).collect()
    }
}
