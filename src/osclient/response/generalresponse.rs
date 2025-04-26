use serde::Deserialize;

// Represents empty body response from the server
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GeneralResponse<T> {
    subsonic_response: T,
}
