mod appconfig;
mod authconfig;
mod behaviourconfig;
mod keybindings;
mod lyricsconfig;
pub mod pathconfig;
mod styleconfig;

use keybindings::KeyBindings;
use std::{collections::HashMap, env};

use color_eyre::Result;
use derive_deref::{Deref, DerefMut};
use lazy_static::lazy_static;
use ratatui::style::Style;
use serde::{de::Deserializer, Deserialize};
use tracing::error;

use crate::{
    app::Mode,
    config::{
        appconfig::AppConfig,
        authconfig::{AuthConfig, UnsafeAuthConfig},
        behaviourconfig::BehaviourConfig,
        lyricsconfig::LyricsConfig,
        pathconfig::PathConfig,
        styleconfig::StyleConfig,
    },
};

#[derive(Clone, Debug, Deserialize)]
pub struct InitState {
    #[serde(default)]
    pub volume: f32,
}

impl Default for InitState {
    fn default() -> Self {
        Self { volume: 0.5 }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default, flatten)]
    pub config: AppConfig,
    #[serde(default)]
    pub keybindings: KeyBindings,
    #[serde(default)]
    pub styles: Styles,
    #[serde(default)]
    pub auth: Option<AuthConfig>,
    #[serde(default)]
    pub unsafe_auth: Option<UnsafeAuthConfig>,
    #[serde(default)]
    pub init_state: InitState,
    #[serde(default)]
    pub lyrics: LyricsConfig,
    #[serde(default)]
    pub behaviour: BehaviourConfig,
}

lazy_static! {
    pub static ref PROJECT_NAME: String = env!("CARGO_CRATE_NAME").to_uppercase().to_string();
}
const CONFIG: &str = include_str!("../.config/config.json5");

impl Config {
    pub fn new(paths: PathConfig) -> Result<Self, config::ConfigError> {
        let default_config: Config = json5::from_str(CONFIG).unwrap();
        let mut builder =
            config::Config::builder().set_default("data_dir", paths.data.to_str().unwrap())?;
        if let Some(c) = paths.config {
            builder = builder.set_default("config_dir", c.to_str().unwrap())?;

            let config_files = [
                ("config.json5", config::FileFormat::Json5),
                ("config.json", config::FileFormat::Json),
                ("config.yaml", config::FileFormat::Yaml),
                ("config.toml", config::FileFormat::Toml),
                ("config.ini", config::FileFormat::Ini),
            ];
            let mut found_config = false;
            for (file, format) in &config_files {
                let source = config::File::from(c.join(file))
                    .format(*format)
                    .required(false);
                builder = builder.add_source(source);
                if c.join(file).exists() {
                    found_config = true;
                }
            }

            if !found_config {
                error!("No configuration file found. Application may not behave as expected");
            }
        };

        let mut cfg: Self = builder.build()?.try_deserialize()?;

        // Add user config on top of the default config
        for (mode, default_bindings) in default_config.keybindings.iter() {
            let user_bindings = cfg.keybindings.entry(*mode).or_default();
            for (key, cmd) in default_bindings.iter() {
                user_bindings
                    .entry(key.clone())
                    .or_insert_with(|| cmd.clone());
            }
        }

        // Add Common keybindings to all other modes
        let common = cfg.keybindings.get(&Mode::Common).cloned();
        if let Some(common_binds) = common {
            for (mode, bindings) in cfg.keybindings.iter_mut() {
                if Mode::Common == *mode {
                    continue;
                };
                for (key, cmd) in common_binds.iter() {
                    bindings.entry(key.clone()).or_insert_with(|| cmd.clone());
                }
            }
        };

        for (mode, default_styles) in default_config.styles.iter() {
            let user_styles = cfg.styles.entry(*mode).or_default();
            for (style_key, style) in default_styles.iter() {
                user_styles.entry(style_key.clone()).or_insert(*style);
            }
        }

        Ok(cfg)
    }
}

#[derive(Clone, Debug, Default, Deref, DerefMut)]
pub struct Styles(pub HashMap<Mode, HashMap<String, Style>>);

impl<'de> Deserialize<'de> for Styles {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let parsed_map = HashMap::<Mode, HashMap<String, String>>::deserialize(deserializer)?;

        let styles = parsed_map
            .into_iter()
            .map(|(mode, inner_map)| {
                let converted_inner_map = inner_map
                    .into_iter()
                    .map(|(str, style)| (str, StyleConfig::parse_style(&style)))
                    .collect();
                (mode, converted_inner_map)
            })
            .collect();

        Ok(Styles(styles))
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use pretty_assertions::assert_eq;
    use ratatui::style::{Color, Modifier};

    use crate::action::Action;

    use super::*;

    #[test]
    fn test_parse_style_default() {
        let style = StyleConfig::parse_style("");
        assert_eq!(style, Style::default());
    }

    #[test]
    fn test_parse_style_foreground() {
        let style = StyleConfig::parse_style("red");
        assert_eq!(style.fg, Some(Color::Indexed(1)));
    }

    #[test]
    fn test_parse_style_background() {
        let style = StyleConfig::parse_style("on blue");
        assert_eq!(style.bg, Some(Color::Indexed(4)));
    }

    #[test]
    fn test_parse_style_modifiers() {
        let style = StyleConfig::parse_style("underline red on blue");
        assert_eq!(style.fg, Some(Color::Indexed(1)));
        assert_eq!(style.bg, Some(Color::Indexed(4)));
    }

    #[test]
    fn test_process_color_string() {
        let (color, modifiers) = StyleConfig::process_color_string("underline bold inverse gray");
        assert_eq!(color, "gray");
        assert!(modifiers.contains(Modifier::UNDERLINED));
        assert!(modifiers.contains(Modifier::BOLD));
        assert!(modifiers.contains(Modifier::REVERSED));
    }

    #[test]
    fn test_parse_color_rgb() {
        let color = StyleConfig::parse_color("rgb123");
        let expected = 16 + 36 + 2 * 6 + 3;
        assert_eq!(color, Some(Color::Indexed(expected)));
    }

    #[test]
    fn test_parse_color_unknown() {
        let color = StyleConfig::parse_color("unknown");
        assert_eq!(color, None);
    }

    #[test]
    fn test_config() -> Result<()> {
        let c = Config::new(PathConfig::default())?;
        let bound_action = c
            .keybindings
            .get(&Mode::Common)
            .unwrap()
            .get(&KeyBindings::parse_key_sequence("<Ctrl-w><q>").unwrap_or_default())
            .unwrap();
        assert!(matches!(bound_action, Action::Quit));
        Ok(())
    }

    #[test]
    fn test_simple_keys() {
        assert_eq!(
            KeyBindings::parse_key_event("a").unwrap(),
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty())
        );

        assert_eq!(
            KeyBindings::parse_key_event("enter").unwrap(),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::empty())
        );

        assert_eq!(
            KeyBindings::parse_key_event("esc").unwrap(),
            KeyEvent::new(KeyCode::Esc, KeyModifiers::empty())
        );
    }

    #[test]
    fn test_with_modifiers() {
        assert_eq!(
            KeyBindings::parse_key_event("ctrl-a").unwrap(),
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL)
        );

        assert_eq!(
            KeyBindings::parse_key_event("alt-enter").unwrap(),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT)
        );

        assert_eq!(
            KeyBindings::parse_key_event("shift-esc").unwrap(),
            KeyEvent::new(KeyCode::Esc, KeyModifiers::SHIFT)
        );
    }

    #[test]
    fn test_multiple_modifiers() {
        assert_eq!(
            KeyBindings::parse_key_event("ctrl-alt-a").unwrap(),
            KeyEvent::new(
                KeyCode::Char('a'),
                KeyModifiers::CONTROL | KeyModifiers::ALT
            )
        );

        assert_eq!(
            KeyBindings::parse_key_event("ctrl-shift-enter").unwrap(),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL | KeyModifiers::SHIFT)
        );
    }

    #[test]
    fn test_reverse_multiple_modifiers() {
        assert_eq!(
            KeyBindings::key_event_to_string(&KeyEvent::new(
                KeyCode::Char('a'),
                KeyModifiers::CONTROL | KeyModifiers::ALT
            )),
            "Ctrl-Alt-a".to_string()
        );
    }

    #[test]
    fn test_invalid_keys() {
        assert!(KeyBindings::parse_key_event("invalid-key").is_err());
        assert!(KeyBindings::parse_key_event("ctrl-invalid-key").is_err());
    }

    #[test]
    fn test_case_insensitivity() {
        assert_eq!(
            KeyBindings::parse_key_event("CTRL-a").unwrap(),
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL)
        );

        assert_eq!(
            KeyBindings::parse_key_event("AlT-eNtEr").unwrap(),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT)
        );
    }
}
