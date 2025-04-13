pub mod query;
pub mod response;

use color_eyre::{eyre, Result};
use query::Query;
use response::Response;
use sunk::Client;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::action::Action;
use crate::queryworker::{query::login::LoginQuery, response::login::LoginResponse};
use crate::trace_dbg;

pub struct QueryWorker {
    client: Option<Client>,
    req_tx: UnboundedSender<Query>,
    req_rx: UnboundedReceiver<Query>,
    action_tx: UnboundedSender<Action>,
    should_quit: bool,
}

impl QueryWorker {
    pub async fn login(q: LoginQuery) -> LoginResponse {
        match q {
            LoginQuery::Login(credentials) => {
                let client = Client::new(
                    credentials.url.as_str(),
                    credentials.username.as_str(),
                    credentials.password.as_str(),
                );
                match client {
                    Ok(c) => match c.ping() {
                        Ok(_) => LoginResponse::Success,
                        Err(_) => LoginResponse::FailedPing,
                    },
                    Err(_) => LoginResponse::Other("Login failed!".to_string()),
                }
            }
        }
    }
    pub async fn run(&mut self) -> Result<()> {
        loop {
            let Some(event) = self.req_rx.recv().await else {
                break;
            };
            match event {
                Query::Stop => self.should_quit = true,
                Query::SetCredentials(creds) => {
                    self.client = Some(
                        Client::new(
                            creds.url.as_str(),
                            creds.username.as_str(),
                            creds.password.as_str(),
                        )
                        .map_err(|e| eyre::eyre!(e))?,
                    );
                }
                Query::Login(login_query) => {
                    let tx = self.action_tx.clone();
                    let _ = tokio::spawn(async move {
                        let res = QueryWorker::login(login_query).await;
                        tx.send(Action::Response(Response::Login(res)))
                    });
                }
            };
            if self.should_quit {
                break;
            }
        }
        Ok(())
    }
}

impl QueryWorker {
    pub fn new(sender: UnboundedSender<Action>) -> Self {
        let (req_tx, req_rx) = mpsc::unbounded_channel();
        Self {
            client: None,
            req_tx,
            req_rx,
            action_tx: sender,
            should_quit: false,
        }
    }
    pub fn get_tx(&self) -> UnboundedSender<Query> {
        self.req_tx.clone()
    }
}
