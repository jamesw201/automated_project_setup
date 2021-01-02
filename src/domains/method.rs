use serde::{Deserialize, Serialize};
use crate::domains::function_signature::{ Dependency, FunctionSignature };

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ParsedMethod {
    pub raw: String,
    pub ast: FunctionSignature,
}

impl ParsedMethod {
    pub fn list_dependencies(&self) -> Vec<&Dependency> {
        self.ast.list_dependencies()
    }
}
