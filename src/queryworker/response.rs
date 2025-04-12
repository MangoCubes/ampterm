use login::LoginResponse;
use serde::{Deserialize, Serialize};
use strum::Display;

pub mod login;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]

pub enum Response {
    Login(LoginResponse),
    GetPlaylist,
}
