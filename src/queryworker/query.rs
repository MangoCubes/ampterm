pub mod login;
use login::{Credentials, LoginQuery};
use serde::{Deserialize, Serialize};

use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Query {
    Stop,
    SetCredentials(Credentials),
    Login(LoginQuery),
}
