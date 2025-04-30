use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
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
impl ToString for OSErrorCode {
    fn to_string(&self) -> String {
        match self {
            OSErrorCode::Generic => String::from("Generic error."),
            OSErrorCode::MissingRequiredParam => String::from("Required parameter is missing."),
            OSErrorCode::ClientMustUpgrade => String::from("Incompatible Subsonic REST protocol version. Client must upgrade."),
            OSErrorCode::ServerMustUpgrade => String::from("Incompatible Subsonic REST protocol version. Server must upgrade."),
            OSErrorCode::WrongUsernameOrPassword => String::from("Wrong username or password."),
            OSErrorCode::TokenNotSupportedForLDAP => String::from("Token authentication not supported for LDAP users."),
            OSErrorCode::AuthNotSupported => String::from("Provided authentication mechanism not supported."),
            OSErrorCode::AuthConflict => String::from("Multiple conflicting authentication mechanisms provided."),
            OSErrorCode::InvalidAPIKey => String::from("Invalid API key."),
            OSErrorCode::NotAuthorised => String::from("User is not authorized for the given operation."),
            OSErrorCode::TrialExpired => String::from("The trial period for the Subsonic server is over. Please upgrade to Subsonic Premium. Visit subsonic.org for details."),
            OSErrorCode::NotFound => String::from("The requested data was not found."),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorData {
    pub code: OSErrorCode,
    pub message: Option<String>,
    #[serde(alias = "helpUrl")]
    pub help_url: Option<String>,
}

impl ToString for ErrorData {
    fn to_string(&self) -> String {
        match self.message.clone() {
            Some(m) => format!("{}", m),
            None => self.code.to_string(),
        }
    }
}
