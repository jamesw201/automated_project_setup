use std::fs;
use std::env;
use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;
extern crate serde_yaml;
use serde::{Deserialize, Serialize};
use exitfailure::ExitFailure;

use tera::Tera;
use tera::Context;

use std::fs::File;
use std::io::prelude::*;

use crate::domains::file::{ UnprocessedFile, ProcessedFile };
use crate::domains::method::ParsedMethod;
use crate::domains::function_signature::{FunctionSignature, Dependency};
use crate::language_interpreter::{ LanguageInterpreter, LanguageInterpreterForUnitTest };

use crate::signature_parser;
use signature_parser::{ root };


#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Schema {
    project_name: String,
    root_directory: String,
    language: String,
    system_types: std::path::PathBuf,
    domain_types: std::path::PathBuf,
    pub files: Vec<UnprocessedFile>,
    workflow: Vec<String>,
}

#[derive(Debug)]
pub struct ParsedSchema {
    project_name: String,
    root_directory: String,
    language: String,
    system_types: std::path::PathBuf,
    domain_types: std::path::PathBuf,
    pub files: Vec<ProcessedFile>,
    workflow: Vec<String>,
    pub templates: tera::Tera,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MockTest {
    pub description: String,
    pub mock_response: String,
}

#[derive(Serialize)]
struct Product {
    name: String
}

impl LanguageInterpreterForUnitTest for MockTest {
    fn as_python(&self, mock_ref: &str, mock_config: &MockConfig, target_ref: &str) -> String {
        let snake_case_description = self.description.replace(" ", "_");    
        format!("def test_{}():\n    {} = MagicMock({})\n    result = {}({})\n    assert 1 == 2", 
            snake_case_description, mock_ref, self.mock_response, target_ref, mock_config.name)
    }
    fn as_javascript(&self, mock_ref: &str, mock_config: &MockConfig, target_ref: &str) -> String {
        format!("it('{}', () => {{}}))", self.description)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MockConfig {
    pub name: String,
    pub imports: Vec<String>,
    pub mock: String,
    pub tests: Vec<MockTest>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MockListing {
    mocks: Vec<MockConfig>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MethodAndMocks {
    pub method: ParsedMethod,
    pub mocks: Vec<MockConfig>,
}
// TODO: provide functionality for:
// [√] creating files
// [√] creating functions
//     [ ] check types exist
// [ ] partially apply dependencies in the main file
// [ ] creating unit tests


// Basic Schema which can return a ParsedSchema
impl Schema {
    pub fn process_schema(&self) -> Result<ParsedSchema, ExitFailure> {
        let processed_files = self.files.iter().map(|file| 
            ProcessedFile::new(file.path.clone(), Self::create_ast(&file))
        ).collect::<Vec<_>>();

        let project_path = std::path::PathBuf::from("./project_repository/templates/python38");
        let full_path = fs::canonicalize(&project_path)?;
        let template_path = full_path.as_path().display().to_string() + "/**/*.hbs";

        let tera = match Tera::new(template_path.as_str()) {
            Ok(t) => {
                t
            },
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };

        Ok(
            ParsedSchema {
                project_name: self.project_name.clone(),
                root_directory: self.root_directory.clone(),
                language: self.language.clone(),
                system_types: self.system_types.clone(),
                domain_types: self.domain_types.clone(),
                files: processed_files,
                workflow: self.workflow.clone(),
                templates: tera,
            }
        )
    }

    fn create_ast(file: &UnprocessedFile) -> Vec<ParsedMethod> {
        file.methods.iter().map(|method| {
            let (_, ast) = root(method.as_str()).unwrap();
            ParsedMethod { raw: method.to_string(), ast }
        }).collect()
    }
}


impl ParsedSchema {
    pub fn generate(&self) -> Result<(), ExitFailure> {
        println!("about to read mock file");
        let mock_listing_path = std::path::PathBuf::from("./project_repository/mocks/python/mock_listings.yaml");
        let mock_listing_file_handler = std::fs::File::open(mock_listing_path)?;
        let mock_listings: MockListing = serde_yaml::from_reader(mock_listing_file_handler)?;
        println!("mock file successfully read");
        
        let key = "PROJECT_SETUP_HOME";
        match env::var_os(key) {
            Some(val) => {
                println!("{}: {:?}", key, val);
                println!("Application root dir: {:?}\n", self.root_directory);

                self.create_main_file();
                self.create_application_files();
                self.create_mocks_file(&mock_listings); // TODO: this needs to return a list of mocks to be included in a mocks files
                self.create_test_files(&mock_listings);
            },
            None => println!("{} is not defined in the environment.", key),
        }

        Ok(())
    }


    // get consolidated list of this schema's dependencies
    fn list_dependencies(&self) -> Vec<&Dependency> {
        self.files.iter().flat_map(|file| file.list_dependencies()).collect()
    }


    pub fn create_mocks_file(&self, mock_listings: &MockListing) -> Result<(), ExitFailure> {
        let mut dependencies: Vec<&Dependency> = self.list_dependencies();
        dependencies.sort_by(|a, b| b.dependency_name.cmp(&a.dependency_name));
        dependencies.dedup();

        let mock_configs: Vec<&MockConfig> = dependencies.into_iter().flat_map(|dependency| {
            mock_listings.mocks.iter().find(|mock| mock.name == dependency.dependency_name)
        }).collect();

        let mock_list: Vec<String> = mock_configs.iter().map(|config| config.mock.clone()).collect();

        let mut context = Context::new();
        context.insert("mock_list", &mock_list);
        let output = self.templates.render("test_mocks.hbs", &context)?;

        self.write_to_file(&output, "tests/mocks.py");

        Ok(())
    }


    fn write_to_file(&self, content: &String, filename: &str) {
        let path_str = format!("{}/{}", self.root_directory, filename);
        let path = Path::new(&path_str);
        let display = path.display();

        let prefix = path.parent().unwrap();
        std::fs::create_dir_all(prefix).unwrap();

        let module_file = match self.language.as_str() {
            "python" => "__init_.py",
            "rust" => "mod.rs",
            _ => "unknown_language",
        }; 
            
        let parent = path.parent().unwrap();
        let file_path = PathBuf::from(parent).join(module_file);
        let init_file_exists = file_path.exists();
        if init_file_exists == false {
            match File::create(&file_path) {
                Err(why) => panic!("couldn't create {}: {}", display, why),
                Ok(file) => file,
            };
        }

        let mut file = match File::create(&path) {
            Err(why) => panic!("couldn't create {}: {}", display, why),
            Ok(file) => file,
        };

        match file.write_all(content.as_bytes()) {
            Err(why) => panic!("couldn't write to {}: {}", display, why),
            Ok(_) => println!("successfully wrote to {}", display),
        }
    }

    fn create_application_files(&self) -> Result<(), ExitFailure> {
        for file in &self.files {
            let functions = file.methods.iter().map(|method| method.ast.as_python()).collect::<Vec<String>>().join("\n");

            // TODO: 
            // [ ] the layout should be dictated by the handlebars template like with the mocks
            // [ ] Types need to be imported with each file
            let path = file.path.as_path().display().to_string();
            let full_path = match self.language.as_str() {
                "python" => format!("{}.py", path),
                "javascript" => format!("{}.js", path),
                _ => format!("{}", path),
            };
            self.write_to_file(&functions, full_path.as_str());
        }

        Ok(())
    }


    fn build_tests_for_methods(&self, method_and_mock: &MethodAndMocks, is_main: bool) -> Vec<String> {
        method_and_mock.mocks.iter().flat_map(|config| {
            let mock_ref: Cow<str> = if is_main {
                "index.requests.get".into() // EXAMPLE
            } else {
                format!("{}.get", &config.name).into()
            };
            let method_name = method_and_mock.method.ast.name.as_str();
            config.tests.iter().map(|test| test.as_python(&mock_ref, config, method_name)).collect::<Vec<String>>()
        }).collect()
    }

    fn retrieve_mock_configs(&self, method: &ParsedMethod, mock_listings: &MockListing) -> MethodAndMocks {
        let dependencies: Vec<String> = method.list_dependencies().iter().map(|dep| dep.dependency_name.clone()).collect();

        let mocks = mock_listings.mocks.iter().flat_map(|mock| {
            if dependencies.contains(&mock.name) {
                Some(   mock.clone())
            } else {
                None
            }
        }).collect();

        MethodAndMocks { method: method.clone(), mocks }
    }


    fn create_test_files(&self, mock_listings: &MockListing) -> Result<(), ExitFailure> {
        for file in &self.files {
            let is_main = file.path.ends_with("main") || file.path.ends_with("index");

            let methods_and_mocks: Vec<MethodAndMocks> = file.methods.iter().map(|method| self.retrieve_mock_configs(&method, mock_listings)).collect();
            let tests: Vec<String> = methods_and_mocks.iter().flat_map(|method_and_mocks| self.build_tests_for_methods(&method_and_mocks, is_main)).collect();

            let combined_file_dependencies = file.list_dependencies();

            let mut imports: Vec<String> = methods_and_mocks.iter().map(|mams| {
                mams.mocks.iter().flat_map(|mock| mock.imports.clone()).collect()
            }).collect();
            // TODO: it should import RequestsMock not requests?
            let joined_dependencies: String = combined_file_dependencies.iter().map(|dep| dep.dependency_name.clone()).collect::<Vec<String>>().join(",");
            // TODO: this is python specific, generalise.
            imports.push(format!("from tests.mocks import {}", joined_dependencies));

            let path = file.path.as_path().display().to_string();
            let test_file_path = path.replace("src", "tests");
            let full_path = match self.language.as_str() {
                "python" => format!("{}.py", test_file_path),
                "javascript" => format!("{}.js", test_file_path),
                _ => format!("{}", test_file_path),
            };
            
            let python_sys_path_assignment = format!(r#"BASE_DIR = os.path.dirname(os.path.abspath(__file__))
sys.path.append(BASE_DIR)
sys.path.insert(0, os.path.join(BASE_DIR, "../{}"))"#, path);
            let sys_path_assignment = if self.language == "python" {
                python_sys_path_assignment.as_str()
            } else {
                ""
            };
            
            // from src.domains.user import User, PersistedUser  # noqa
            let dot_separated_path: String = path.split("/").collect::<Vec<&str>>().join(".");
            let method_names: String = file.methods.iter().map(|method| method.ast.name.clone()).collect::<Vec<String>>().join(", ");

            let local_imports: Vec<String> = vec![format!("from {} import {}", dot_separated_path, method_names)];

            let mut context = Context::new();
            context.insert("imports", &imports);
            context.insert("local_imports", &local_imports);
            context.insert("sys_path_assignment", sys_path_assignment);
            context.insert("tests", &tests);
            let output = self.templates.render("test.hbs", &context)?;
            
            self.write_to_file(&output, full_path.as_str());
        }

        // create separate test_main.py which requires all mocks
        Ok(())
    }


    fn create_main_file(&self) -> Result<(), ExitFailure> {
        let application_files = vec!["application files", "apfile"];
        let dependencies = vec!["dependencies", "dep2"];
        let functions_with_side_effects = vec!["functions_with_side_effects", "sideeffect"];
        
        let mut context = Context::new();
        context.insert("application_files", &application_files);
        context.insert("dependencies", &dependencies);
        context.insert("functions_with_side_effects", &functions_with_side_effects);
        context.insert("workflow", &self.workflow);
        
        let output = self.templates.render("main.hbs", &context)?;
        
        let full_path = match self.language.as_str() {
            "python" => "src/main.py",
            "javascript" => "src/index.js",
            _ => "unknown_language",
        };
        self.write_to_file(&output, full_path);

        Ok(())
    }
}
