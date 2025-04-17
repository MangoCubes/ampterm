mod login;
// mod mainscreen;
mod loading;

use std::sync::Mutex;

use color_eyre::Result;
use loading::Loading;
use login::Login;
use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{
    action::Action,
    config::Config,
    queryworker::{
        query::{
            login::{Credentials, LoginQuery},
            Query,
        },
        response::{login::LoginResponse, Response},
    },
    tui::Event,
};

enum State {
    // Credentials have been loaded from the config, and the home component is showing Loading component
    ConfigLogin,
    // Either the config credentials have been valid, or not found. Currently showing Login component
    Login,
    // Login has been successful
    Main,
}

pub struct Home {
    action_tx: UnboundedSender<Action>,
    component: Box<dyn Component>,
    state: Mutex<State>,
}

impl Home {
    pub fn new(action_tx: UnboundedSender<Action>, config: Config) -> Self {
        let auth = config.clone().auth;
        let config_creds = if let Some(creds) = auth {
            Some(Credentials {
                url: todo!(),
                username: todo!(),
                password: todo!(),
                legacy: config.config.use_legacy_auth,
            })
        } else {
            match config.clone().unsafe_auth {
                Some(unsafe_creds) => Some(Credentials {
                    url: unsafe_creds.url,
                    username: unsafe_creds.username,
                    password: unsafe_creds.password,
                    legacy: config.config.use_legacy_auth,
                }),
                None => None,
            }
        };
        let config_has_creds;
        let comp: Box<dyn Component> = match config_creds {
            Some(creds) => {
                config_has_creds = true;
                let query_creds = creds.clone();
                let action = Action::Query(Query::Login(LoginQuery::Login(Credentials::new(
                    query_creds.url,
                    query_creds.username,
                    query_creds.password,
                    config.config.use_legacy_auth,
                ))));
                match action_tx.send(action) {
                    Ok(_) => Box::new(Loading::new(creds.url, creds.username)),
                    Err(_) => Box::new(Login::new(action_tx.clone(), config)),
                }
            }
            None => {
                config_has_creds = false;
                Box::new(Login::new(action_tx.clone(), config))
            }
        };
        Self {
            action_tx,
            component: comp,
            state: Mutex::new(if config_has_creds {
                State::ConfigLogin
            } else {
                State::Login
            }),
        }
    }
}

impl Component for Home {
    fn handle_events(&mut self, event: Event) -> Result<Option<Action>> {
        if let Some(action) = self.component.handle_events(event.clone())? {
            self.action_tx.send(action)?;
        }
        Ok(None)
    }
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Some(action) = self.component.update(action.clone())? {
            self.action_tx.send(action)?;
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        if let Err(err) = self.component.draw(frame, area) {
            self.action_tx.send(Action::Error(err.to_string()))?;
        }
        Ok(())
    }
}
