use serde::Deserialize;

// Represents empty body response from the server
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Wrapper<T> {
    pub subsonic_response: T,
}
