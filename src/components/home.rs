mod loading;
mod login;
mod mainscreen;

use color_eyre::Result;
use loading::Loading;
use login::Login;
use mainscreen::MainScreen;
use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{
    action::{ping::PingResponse, Action},
    config::Config,
    noparams::NoParams,
    queryworker::query::{setcredential::Credential, Query},
    tui::Event,
};

pub struct Home {
    action_tx: UnboundedSender<Action>,
    component: Box<dyn NoParams>,
    config_has_creds: bool,
    config: Config,
}

impl Home {
    pub fn new(action_tx: UnboundedSender<Action>, config: Config) -> Self {
        let auth = config.clone().auth;
        let config_creds = if let Some(creds) = auth {
            Some(Credential::Password {
                url: todo!(),
                secure: todo!(),
                username: todo!(),
                password: todo!(),
                legacy: todo!(),
            })
        } else {
            match config.clone().unsafe_auth {
                Some(unsafe_creds) => Some(Credential::Password {
                    url: unsafe_creds.url,
                    username: unsafe_creds.username,
                    password: unsafe_creds.password,
                    legacy: config.config.use_legacy_auth,
                    secure: true,
                }),
                None => None,
            }
        };
        let config_has_creds;
        let comp: Box<dyn NoParams> = match config_creds {
            Some(creds) => {
                config_has_creds = true;
                let url = creds.get_url();
                let username = creds.get_username();
                let action = Action::Query(Query::SetCredential(creds));
                let _ = action_tx.send(action);
                let _ = action_tx.send(Action::Query(Query::Ping));
                Box::new(Loading::new(url, username))
            }
            None => {
                config_has_creds = false;
                Box::new(Login::new(action_tx.clone(), config.clone()))
            }
        };
        Self {
            action_tx,
            component: comp,
            config_has_creds,
            config,
        }
    }
}

impl Component for Home {
    fn handle_events(&mut self, event: Event) -> Result<Option<Action>> {
        if let Some(action) = self.component.handle_events(event)? {
            self.action_tx.send(action)?;
        }
        Ok(None)
    }
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        // Child component can change in two cases:
        // 1. Login is successful regardless of the current child component
        // 2. Login with the config credentials fails
        if let Action::Ping(res) = &action {
            match res {
                PingResponse::Success => {
                    // Switch child component to MainScreen
                    self.component = Box::new(MainScreen::new(self.action_tx.clone()));
                    return Ok(None);
                }
                PingResponse::Failure(_) => {
                    if self.config_has_creds {
                        self.config_has_creds = false;
                        // Switch child component to Login
                        self.component =
                            Box::new(Login::new(self.action_tx.clone(), self.config.clone()));
                        return Ok(None);
                    }
                }
            };
        };
        if let Some(action) = self.component.update(action)? {
            self.action_tx.send(action)?;
        }
        Ok(None)
    }
}

impl NoParams for Home {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        if let Err(err) = self.component.draw(frame, area) {
            self.action_tx.send(Action::Error(err.to_string()))?;
        }
        Ok(())
    }
}
