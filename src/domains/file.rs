use serde::{Deserialize, Serialize};
use crate::domains::method::ParsedMethod;
use crate::domains::function_signature::Dependency;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct UnprocessedFile {
    pub path: std::path::PathBuf,
    pub methods: Vec<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ProcessedFile {
    pub path: std::path::PathBuf,
    pub methods: Vec<ParsedMethod>,
}

impl ProcessedFile {
    pub fn new(path: std::path::PathBuf, methods: Vec<ParsedMethod>) -> ProcessedFile {
        ProcessedFile { path, methods }
    }

    pub fn list_dependencies(&self) -> Vec<&Dependency> {
        self.methods.iter().flat_map(|method| method.ast.list_dependencies()).collect()
    }

    // python imports need you to set the syspath
    // then split the path after src and join back up with full-stops

    // relative imports can be worked out from the following:
    // - file1 path
    // - file1 method name
    // - file2 path
}
