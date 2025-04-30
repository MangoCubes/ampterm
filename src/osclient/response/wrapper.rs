use serde::Deserialize;
use std::fmt::Debug;

// Response has a layer of JSON around it, this represents that
#[derive(Debug, Deserialize)]
pub struct Wrapper<T: Debug> {
    #[serde(alias = "subsonic-response")]
    pub subsonic_response: T,
}
