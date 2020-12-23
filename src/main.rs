extern crate serde_yaml;
use serde::{Deserialize, Serialize};
use exitfailure::ExitFailure;

mod signature_parser;
use signature_parser::root;

mod domains;
use crate::domains::schema::Schema;
use domains::cli::Cli;


fn main() -> Result<(), ExitFailure> {
    let schema_path = std::env::args().nth(1).expect("no path given");
    let args = Cli {
        schema_path: std::path::PathBuf::from(schema_path),
    };
    let schema_file_handler = std::fs::File::open(&args.schema_path)?;
    let schema_file: Schema = serde_yaml::from_reader(schema_file_handler)?;

    let processed_schema = schema_file.process_schema();
    processed_schema.retrieve_mocks();

    // TODO:
    // [√] write parser for function signatures
    // [ ] create handlebars templates for code generation
    // [ ] lookup mocks for dependencies

    // println!("schema_file: {:?}", schema_file);
    Ok(())
}
