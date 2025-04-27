use serde::Deserialize;

// Response has a layer of JSON around it, this represents that
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Wrapper<T> {
    pub subsonic_response: T,
}
