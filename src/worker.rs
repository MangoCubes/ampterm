use sunk::Client;

use crate::action::login::{LoginQuery, LoginResponse};

pub struct Worker {
    client: Option<Client>,
}

impl Worker {
    pub fn login(&mut self, q: LoginQuery) -> LoginResponse {
        match q {
            LoginQuery::Login(credentials) => {
                let client = Client::new(
                    credentials.url.as_str(),
                    credentials.username.as_str(),
                    credentials.password.as_str(),
                );
                match client {
                    Ok(client) => {
                        self.client = Some(client);
                        LoginResponse::Success
                    }
                    Err(_) => LoginResponse::Other("Login failed!".to_string()),
                }
            }
        }
    }
}

impl Default for Worker {
    fn default() -> Self {
        Self { client: None }
    }
}
