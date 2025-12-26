use derive_deref::Deref;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Deref)]
pub struct MediaID(String);

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Deref)]
pub struct PlaylistID(String);
