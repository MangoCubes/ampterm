use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum OSErrorCode {
    Generic = 0,
    MissingRequiredParam = 10,
    ClientMustUpgrade = 20,
    ServerMustUpgrade = 30,
    WrongUsernameOrPassword = 40,
    TokenNotSupportedForLDAP = 41,
    AuthNotSupported = 42,
    AuthConflict = 43,
    InvalidAPIKey = 44,
    NotAuthorised = 50,
    TrialExpired = 60,
    NotFound = 70,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorData {
    pub code: OSErrorCode,
    pub message: Option<String>,
    pub help_url: Option<String>,
}
