extern crate serde_yaml;
extern crate serde_json;
extern crate tera;
use exitfailure::ExitFailure;

mod signature_parser;
mod domains;
mod language_interpreter;

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
    match processed_schema {
        Ok(result) => result.generate(),
        Err(err) => Err(err),
    }
}
