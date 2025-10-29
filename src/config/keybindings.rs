use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use derive_deref::{Deref, DerefMut};
use serde::{Deserialize, Deserializer};

use crate::{action::Action, app::Mode, trace_dbg};

#[derive(Clone, Debug, Default, Deref, DerefMut)]
pub struct KeyBindings(pub HashMap<Mode, HashMap<Vec<KeyEvent>, Action>>);

impl<'de> Deserialize<'de> for KeyBindings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let parsed_map = HashMap::<Mode, HashMap<String, Action>>::deserialize(deserializer)?;

        let keybindings = parsed_map
            .into_iter()
            .map(|(mode, inner_map)| {
                let converted_inner_map = inner_map
                    .into_iter()
                    .map(|(key_str, cmd)| (Self::parse_key_sequence(&key_str).unwrap(), cmd))
                    .collect();
                (mode, converted_inner_map)
            })
            .collect();

        Ok(KeyBindings(trace_dbg!(keybindings)))
    }
}

impl KeyBindings {
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
