mod loading;
mod login;
mod mainscreen;

use color_eyre::Result;
use loading::Loading;
use login::Login;
use mainscreen::MainScreen;
use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    compid,
    components::traits::component::Component,
    config::{get_config_dir, Config},
    queryworker::query::{
        ping::PingResponse, setcredential::Credential, QueryType, ResponseType, ToQueryWorker,
    },
    tui::Event,
};

enum Comp {
    Main(MainScreen),
    Loading(Loading),
    Login(Login),
}

pub struct Home {
    action_tx: UnboundedSender<Action>,
    component: Comp,
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
        let comp: Comp = match config_creds {
            Some(creds) => {
                let url = creds.get_url();
                let username = creds.get_username();
                let _ = action_tx.send(Action::Multiple(vec![
                    Some(Action::ToQueryWorker(ToQueryWorker::new(
                        QueryType::SetCredential(creds),
                    ))),
                    Some(Action::ToQueryWorker(ToQueryWorker::new(QueryType::Ping))),
                ]));
                Comp::Loading(Loading::new(url, username))
            }
            None => Comp::Login(Login::new(
                action_tx.clone(),
                Some(vec![
                    "No credentials detected in the config.".to_string(),
                    format!("(Loaded config from {:?})", get_config_dir()),
                ]),
                config.clone(),
            )),
        };
        Self {
            action_tx,
            component: comp,
            config,
        }
    }
}

impl Component for Home {
    fn handle_events(&mut self, event: Event) -> Result<Option<Action>> {
        if let Comp::Main(c) = &mut self.component {
            c.handle_events(event)
        } else {
            Ok(None)
        }
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        match &mut self.component {
            Comp::Loading(c) => c.draw(frame, area),
            Comp::Login(c) => c.draw(frame, area),
            Comp::Main(c) => c.draw(frame, area),
        }
    }
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        // Child component can change in two cases:
        // 1. Login is successful regardless of the current child component
        // 2. Login with the config credentials fails
        if let Action::FromQueryWorker(res) = &action {
            if let ResponseType::Ping(pr) = &res.res {
                match pr {
                    PingResponse::Success => {
                        // Switch child component to MainScreen
                        self.component = Comp::Main(MainScreen::new(self.action_tx.clone()));
                        return Ok(None);
                    }
                    PingResponse::Failure(err) => {
                        if let Comp::Loading(l) = &self.component {
                            // Switch child component to Login
                            self.component = Comp::Login(Login::new(
                                self.action_tx.clone(),
                                Some(vec![
                                    "Failed to query the server with the given credentials!"
                                        .to_string(),
                                    format!("Error: {}", err),
                                ]),
                                self.config.clone(),
                            ));
                            return Ok(None);
                        }
                    }
                }
            };
        };
        if let Comp::Main(c) = &mut self.component {
            c.update(action)
        } else {
            Ok(None)
        }
    }
}
