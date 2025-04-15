mod login;
// mod mainscreen;
mod loading;

use color_eyre::Result;
use loading::Loading;
use login::Login;
use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{
    action::Action,
    config::Config,
    queryworker::query::{
        login::{Credentials, LoginQuery},
        Query,
    },
    tui::Event,
};

pub struct Home {
    action_tx: UnboundedSender<Action>,
    component: Box<dyn Component>,
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
        let comp: Box<dyn Component> = match config_creds {
            Some(creds) => {
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
            None => Box::new(Login::new(action_tx.clone(), config)),
        };
        Self {
            action_tx,
            component: comp,
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
