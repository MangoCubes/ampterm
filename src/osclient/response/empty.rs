use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum Empty {
    Ok,
    Failed,
}
