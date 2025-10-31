use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Default)]
pub struct AuthConfig {
    #[serde(default)]
    pub url_command: String,
    #[serde(default)]
    pub username_command: String,
    #[serde(default)]
    pub password_command: String,
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
