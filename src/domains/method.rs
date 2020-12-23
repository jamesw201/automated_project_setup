use serde::{Deserialize, Serialize};
use crate::signature_parser::FunctionSignature;

#[derive(Serialize, Deserialize, Debug)]
pub struct ParsedMethod {
    pub raw: String,
    pub ast: FunctionSignature,
}
