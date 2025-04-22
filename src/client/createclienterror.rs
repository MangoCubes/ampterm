use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct CreateClientError;

impl Error for CreateClientError {}

impl Display for CreateClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
