use serde::{Deserialize, Serialize};

use super::errordata::ErrorData;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "status")]
pub enum Empty {
    #[serde(alias = "ok")]
    Ok,
    #[serde(alias = "failed")]
    Failed { error: ErrorData },
}
