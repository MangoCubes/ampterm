use serde::{Deserialize, Serialize};

use super::oserror::OSError;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "status")]
pub enum Empty {
    #[serde(alias = "ok")]
    Ok,
    #[serde(alias = "failed")]
    Failed { error: OSError },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AlwaysError {
    pub error: OSError,
}
