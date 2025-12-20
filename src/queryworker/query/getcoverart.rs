use bytes::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct CoverID(pub String);

#[derive(Debug, Clone, PartialEq)]
pub enum GetCoverArtResponse {
    Success(Bytes),
    Failure { msg: String },
}
