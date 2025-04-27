use serde::{Deserialize, Serialize};

use super::errordata::ErrorData;

#[derive(Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum Empty {
    Ok,
    Failed { error: ErrorData },
}
