mod loading;
mod login;
mod mainscreen;

use std::process::Command;

use color_eyre::Result;
use crossterm::event::KeyEvent;
use loading::Loading;
use login::Login;
use mainscreen::MainScreen;
use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    components::traits::{fullcomp::FullComp, ontick::OnTick, renderable::Renderable},
    config::Config,
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{setcredential::Credential, ResponseType, ToQueryWorker},
    },
    tui::Event,
};

enum Comp {
    Main(MainScreen),
    Loading(Loading),
    Login(Login),
}

pub struct Home {
    component: Comp,
    config: Config,
}

impl OnTick for Home {
    fn on_tick(&mut self) {
        if let Comp::Main(main_screen) = &mut self.component {
            main_screen.on_tick();
        };
    }
}

impl Home {
    pub fn new(config: Config) -> (Self, Vec<Action>) {
        let auth = config.clone().auth;
        let config_creds = if let Some(creds) = auth {
            fn run_cmd(cmd: &String) -> Result<String> {
                let exec = Command::new("sh").arg("-c").arg(cmd).output()?;
                let stdout = String::from_utf8_lossy(&exec.stdout);
                Ok(stdout.trim().to_string())
            }
            if let Ok(url) = run_cmd(&creds.url) {
                if let Ok(username) = run_cmd(&creds.username) {
                    if let Ok(password) = run_cmd(&creds.password) {
                        Some(Credential::Password {
                            url,
                            secure: true,
                            username,
                            password,
                            legacy: config.config.use_legacy_auth,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
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
        let (comp, actions): (Comp, Vec<Action>) = match config_creds {
            Some(creds) => {
                let url = creds.get_url();
                let username = creds.get_username();
                (
                    Comp::Loading(Loading::new(url, username)),
                    vec![
                        Action::ToQueryWorker(ToQueryWorker::new(HighLevelQuery::SetCredential(
                            creds,
                        ))),
                        Action::ToQueryWorker(ToQueryWorker::new(
                            HighLevelQuery::CheckCredentialValidity,
                        )),
                    ],
                )
            }
            None => {
                let (comp, action) = Login::new(
                    Some(vec![
                        "No credentials detected in the config.".to_string(),
                        format!("(Loaded config from {:?})", Config::get_config_dir()),
                    ]),
                    config.clone(),
                );
                (Comp::Login(comp), vec![action])
            }
        };
        (
            Self {
                component: comp,
                config,
            },
            actions,
        )
    }
}

impl Renderable for Home {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        match &mut self.component {
            Comp::Loading(c) => c.draw(frame, area),
            Comp::Login(c) => c.draw(frame, area),
            Comp::Main(c) => c.draw(frame, area),
        }
    }
}

impl FullComp for Home {
    fn handle_events(&mut self, event: Event) -> Result<Option<Action>> {
        if let Comp::Main(c) = &mut self.component {
            c.handle_events(event)
        } else {
            Ok(None)
        }
    }
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        match &mut self.component {
            Comp::Login(login) => login.handle_key_event(key),
            _ => Ok(None),
        }
    }
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        // Child component can change in two cases:
        // 1. Login is successful regardless of the current child component
        // 2. Login with the config credentials fails
        if let Action::FromQueryWorker(res) = &action {
            if let ResponseType::Ping(pr) = &res.res {
                match pr {
                    Ok(()) => {
                        // Switch child component to MainScreen
                        let (comp, actions) = MainScreen::new(self.config.clone());
                        self.component = Comp::Main(comp);
                        return Ok(Some(actions));
                    }
                    Err(err) => {
                        if let Comp::Loading(l) = &self.component {
                            // Switch child component to Login
                            let (comp, action) = Login::new(
                                Some(vec![
                                    "Failed to query the server with the given credentials!"
                                        .to_string(),
                                    format!("Error: {}", err),
                                ]),
                                self.config.clone(),
                            );
                            self.component = Comp::Login(comp);
                            return Ok(Some(action));
                        }
                    }
                }
            };
        };
        match &mut self.component {
            Comp::Main(main_screen) => main_screen.update(action),
            Comp::Loading(_loading) => Ok(None),
            Comp::Login(login) => login.update(action),
        }
    }
}
