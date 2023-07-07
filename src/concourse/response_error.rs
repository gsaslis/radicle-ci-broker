use std::error::Error;
use std::fmt::{Display, Formatter};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Warning {
    #[serde(rename = "type")]
    pub r#type: String,
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct ResponseError {
    pub errors: Vec<String>,
    pub warnings: Option<Vec<Warning>>,
}

impl Error for ResponseError {}

impl Display for ResponseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ errors: {:?}, warnings: {:?} }}", self.errors, self.warnings)
    }
}

