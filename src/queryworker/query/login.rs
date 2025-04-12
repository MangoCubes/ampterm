use serde::{Deserialize, Serialize};

use strum::Display;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Credentials {
    pub url: String,
    pub username: String,
    pub password: String,
}

impl Credentials {
    pub fn new(url: String, username: String, password: String) -> Self {
        Self {
            url,
            username,
            password,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum LoginQuery {
    // Login component has request Login action
    Login(Credentials),
}
