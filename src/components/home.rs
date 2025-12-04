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

use crate::{
    action::action::{Action, Mode, QueryAction, TargetedAction},
    components::traits::{
        handleaction::HandleAction, handlekeyseq::PassKeySeq, handlemode::HandleMode,
        handlequery::HandleQuery, handleraw::HandleRaw, ontick::OnTick, renderable::Renderable,
    },
    config::{pathconfig::PathConfig, Config},
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{setcredential::Credential, ResponseType, ToQueryWorker},
    },
};

use super::traits::handlekeyseq::{ComponentKeyHelp, KeySeqResult};

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

impl PassKeySeq for Home {
    fn handle_key_seq(&mut self, keyseq: &Vec<KeyEvent>) -> Option<KeySeqResult> {
        match &mut self.component {
            Comp::Main(main_screen) => main_screen.handle_key_seq(keyseq),
            Comp::Login(_) | Comp::Loading(_) => Some(KeySeqResult::NoActionNeeded),
        }
    }

    fn get_help(&self) -> Vec<ComponentKeyHelp> {
        match &self.component {
            Comp::Main(main_screen) => main_screen.get_help(),
            _ => vec![],
        }
    }
}

impl HandleAction for Home {
    fn handle_action(&mut self, action: TargetedAction) -> Option<Action> {
        match &mut self.component {
            Comp::Main(main_screen) => main_screen.handle_action(action),
            Comp::Login(_) | Comp::Loading(_) => None,
        }
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
                        Action::Query(QueryAction::ToQueryWorker(ToQueryWorker::new(
                            HighLevelQuery::SetCredential(creds),
                        ))),
                        Action::Query(QueryAction::ToQueryWorker(ToQueryWorker::new(
                            HighLevelQuery::CheckCredentialValidity,
                        ))),
                    ],
                )
            }
            None => {
                let comp = Login::new(Some(vec![
                    "No credentials detected in the config.".to_string(),
                    format!("(Loaded config from {:?})", PathConfig::get_config_dir()),
                ]));
                (Comp::Login(comp), vec![])
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
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        match &mut self.component {
            Comp::Loading(c) => c.draw(frame, area),
            Comp::Login(c) => c.draw(frame, area),
            Comp::Main(c) => c.draw(frame, area),
        }
    }
}

impl HandleRaw for Home {
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Action> {
        match &mut self.component {
            Comp::Login(login) => login.handle_key_event(key),
            _ => None,
        }
    }
}

impl HandleMode for Home {
    fn handle_mode(&mut self, mode: Mode) {
        if let Comp::Main(main_screen) = &mut self.component {
            main_screen.handle_mode(mode);
        }
    }
}

impl HandleQuery for Home {
    fn handle_query(&mut self, action: QueryAction) -> Option<Action> {
        match &mut self.component {
            Comp::Main(main_screen) => main_screen.handle_query(action),
            Comp::Login(_) | Comp::Loading(_) => {
                // Child component can change in two cases:
                // 1. Login is successful regardless of the current child component
                // 2. Login with the config credentials fails
                if let QueryAction::FromQueryWorker(res) = &action {
                    if let ResponseType::Ping(pr) = &res.res {
                        match pr {
                            Ok(()) => {
                                // Switch child component to MainScreen
                                let (comp, actions) = MainScreen::new(self.config.clone());
                                self.component = Comp::Main(comp);
                                return Some(actions);
                            }
                            Err(err) => {
                                if let Comp::Loading(_) = &self.component {
                                    // Switch child component to Login
                                    self.component = Comp::Login(Login::new(Some(vec![
                                        "Failed to query the server with the given credentials!"
                                            .to_string(),
                                        format!("Error: {}", err),
                                    ])));
                                    return None;
                                }
                            }
                        }
                    };
                };
                None
            }
        }
    }
}
