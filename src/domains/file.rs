use serde::{Deserialize, Serialize};
use crate::domains::method::ParsedMethod;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct File {
    pub path: std::path::PathBuf,
    pub methods: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessedFile {
    pub path: std::path::PathBuf,
    pub methods: Vec<ParsedMethod>,
}

impl ProcessedFile {
    pub fn new(path: std::path::PathBuf, methods: Vec<ParsedMethod>) -> ProcessedFile {
        ProcessedFile { path, methods }
    }
}