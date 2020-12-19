extern crate serde_yaml;
use serde::{Deserialize, Serialize};
use exitfailure::ExitFailure;

mod signature_parser;


struct Cli {
    schema_path: std::path::PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
struct File {
    path: std::path::PathBuf,
    methods: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Schema {
    project_name: String,
    root_directory: String,
    language: String,
    dependencies: Vec<String>,
    system_types: std::path::PathBuf,
    domain_types: std::path::PathBuf,
    files: Vec<File>,
    workflow: Vec<String>,
}


fn main() -> Result<(), ExitFailure> {
    let schema_path = std::env::args().nth(1).expect("no path given");
    let args = Cli {
        schema_path: std::path::PathBuf::from(schema_path),
    };
    let schema_file_handler = std::fs::File::open(&args.schema_path)?;
    let schema_file: Schema = serde_yaml::from_reader(schema_file_handler)?;

    // TODO:
    // [ ] write parser for function signatures
    // [ ] create handlebars templates for code generation
    // [ ] lookup mocks for dependencies

    println!("schema_file: {:?}", schema_file);
    Ok(())
}
