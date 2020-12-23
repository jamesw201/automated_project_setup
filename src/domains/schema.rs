extern crate serde_yaml;
use serde::{Deserialize, Serialize};
use exitfailure::ExitFailure;
use crate::domains::file::{ File, ProcessedFile };
use crate::domains::method::ParsedMethod;

use crate::signature_parser;
use signature_parser::{ root, Dependency };

use handlebars::Handlebars;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Schema {
    project_name: String,
    root_directory: String,
    language: String,
    system_types: std::path::PathBuf,
    domain_types: std::path::PathBuf,
    pub files: Vec<File>,
    workflow: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ParsedSchema {
    project_name: String,
    root_directory: String,
    language: String,
    system_types: std::path::PathBuf,
    domain_types: std::path::PathBuf,
    pub files: Vec<ProcessedFile>,
    workflow: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct MockConfig {
    name: String,
    imports: Vec<String>,
    path: String,
    tests: std::path::PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MockListing {
    mocks: Vec<MockConfig>,
}

// TODO: provide functionality for:
// [ ] creating files
// [ ] creating functions
//     [ ] check types exist
// [ ] partially apply dependencies in the main file
// [ ] creating unit tests


// Basic Schema which can return a ParsedSchema
impl Schema {
    pub fn process_schema(&self) -> ParsedSchema {
        let processed_files = self.files.iter().map(|file| 
            ProcessedFile::new(file.path.clone(), Self::create_ast(&file))
        ).collect::<Vec<_>>();

        ParsedSchema {
            project_name: self.project_name.clone(),
            root_directory: self.root_directory.clone(),
            language: self.language.clone(),
            system_types: self.system_types.clone(),
            domain_types: self.domain_types.clone(),
            files: processed_files,
            workflow: self.workflow.clone(),
        }
    }

    fn create_ast(file: &File) -> Vec<ParsedMethod> {
        file.methods.iter().map(|method| {
            println!("method: {:?}", method);
            let (_, ast) = root(method.as_str()).unwrap();
            println!("AST: {:?}", ast);
            println!("");
            // Should we get the Mocks here and add them to the ParsedMethod?
            ParsedMethod { raw: method.to_string(), ast }
        }).collect()
    }
}


impl ParsedSchema {
    pub fn generate(&self) {
        self.create_files();
        // let parsed_methods = self.files.into_iter().map(|file| file.methods).collect::<Vec<_>>();
    }

    // get consolidated list of this schema's dependencies
    fn list_dependencies(&self) -> Vec<&Dependency> {
        let parsed_methods = self.files.iter().flat_map(|file| &file.methods).collect::<Vec<_>>();
        println!("all methods length: {:?}", parsed_methods.len());
        parsed_methods.into_iter().flat_map(|method| method.ast.list_dependencies()).collect()
    }

    pub fn retrieve_mocks(&self) -> Result<(), ExitFailure> {
        let mock_listing_path = std::path::PathBuf::from("./project_repository/mocks/python/mock_listings.yaml");
        let mock_listing_file_handler = std::fs::File::open(mock_listing_path)?;
        let mock_listings: MockListing = serde_yaml::from_reader(mock_listing_file_handler)?;
        println!("{:?}", mock_listings);

        let mut handlebars = Handlebars::new();
        
        handlebars.register_template_string("mock_template", include_str!("../../project_repository/templates/python38/test_mocks.hbs"))?;

        let dependencies: Vec<&Dependency> = self.list_dependencies();
        println!("dependencies: {:?}", dependencies);
        
        let mock_configs: Vec<&MockConfig> = dependencies.into_iter().flat_map(|dependency| {
            mock_listings.mocks.iter().find(|mock| mock.name == dependency.dependency_name)
        }).collect();
        println!("mock_configs: {:?}", mock_configs);

        for config in &mock_configs {
            let path = format!("./project_repository/mocks/python/{}", &config.path.as_str());
            let template: String = std::fs::read_to_string(path)?.parse()?;
            handlebars.register_template_string(config.name.as_str(), template)?;
        }

        let mock_list: Vec<String> = mock_configs.into_iter().flat_map(|config| {
            let data: HashMap<String, String> = HashMap::new();
            handlebars.render(config.name.as_str(), &data)
        }).collect();

        println!("mock_list: {:?}", mock_list);

        let mut mock_hash: HashMap<&str, Vec<String>> = HashMap::new();
        mock_hash.insert("mock_list", mock_list);
        println!("{}", handlebars.render("mock_template", &mock_hash)?);
        
        Ok(())
    }

    fn create_files(&self) {
        // Dependencies:
        // - dictate which mocks need to be created
        // - which mocks are needed for a given test
        // - what to partially apply 
        // - which standard tests do we need to have for a given dependency
        
        // create mocks file
        // create test files [need to know about mocks, which mocks are needed for each test]
        // create application files
        // create main file [need to know about dependencies, which need to be declared and partially applied]
        self.retrieve_mocks(); // TODO: this needs to return a list of mocks to be included in a mocks file
    }

    fn create_mocks_file(&self) {
        // get the consolidate list of all dependencies in all application files
        // match that list to existing mocks
        // put those mocks in the mocks files and alert the User about any dependencies for which they do not have an existing mock
    }

    fn create_application_files() {
        // generate language specific functions from the ASTs
    }

    fn create_test_files() {
        // import the mocks that are required
        // create standard tests for each dependency
    }

    fn create_main_file() {
        // import all functions from application files
        // loop through all application files 
        //      loop through all functions for each file
        //          return pairs of (file, dependency)
        // create consolidated list [Dependency]
        // declare dependencies
        // create partially applied versions of all functions that need a dependency
        // create workflow of functions/partiallyAppliedFunctions
    }
}
