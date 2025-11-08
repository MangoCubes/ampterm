use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Default)]
pub struct AuthConfig {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
}

// Unsafe settings, but in case you need these
#[derive(Clone, Debug, Deserialize, Default)]
pub struct UnsafeAuthConfig {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
}
