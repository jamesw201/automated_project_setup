use serde::{Deserialize, Serialize};
use crate::language_interpreter::LanguageInterpreter;


#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ApplicationType {
    pub type_name: String,
}

impl LanguageInterpreter for ApplicationType {
    fn as_python(&self) -> String {
        self.type_name.clone()
    }
    fn as_javascript(&self) -> String {
        self.type_name.clone()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ApplicationParentType {
    pub type_name: String,
    pub children: Vec<ParameterType>,
}

impl LanguageInterpreter for ApplicationParentType {
    fn as_python(&self) -> String {
        let children = self.children.iter().map(|child| child.as_python()).collect::<Vec<String>>().join(", ");
        format!("{}[{}]", self.type_name, children)
    }
    fn as_javascript(&self) -> String {
        self.type_name.clone()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Dependency {
    pub dependency_name: String,
}

impl LanguageInterpreter for Dependency {
    fn as_python(&self) -> String {
        self.dependency_name.clone()
    }
    fn as_javascript(&self) -> String {
        self.dependency_name.clone()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum ParameterType {
    ApplicationType(ApplicationType),
    ApplicationParentType(ApplicationParentType),
    Dependency(Dependency),
}

impl LanguageInterpreter for ParameterType {
    fn as_python(&self) -> String {
        match self {
            Self::ApplicationType(value) => value.as_python(),
            Self::ApplicationParentType(value) => value.as_python(),
            Self::Dependency(value) => value.as_python(),
        }
    }
    fn as_javascript(&self) -> String {
        match self {
            Self::ApplicationType(value) => value.as_javascript(),
            Self::ApplicationParentType(value) => value.as_javascript(),
            Self::Dependency(value) => value.as_javascript(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct FunctionParameter {
    pub name: String,
    pub ptype: ParameterType,
}

impl LanguageInterpreter for FunctionParameter {
    fn as_python(&self) -> String {
        format!("{}: {}", self.name, self.ptype.as_python())
    }
    fn as_javascript(&self) -> String {
        format!("{}: {}", self.name, self.ptype.as_javascript())
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub input: Vec<FunctionParameter>,
    pub output: ParameterType,
}

impl LanguageInterpreter for FunctionSignature {
    fn as_python(&self) -> String {
        let parameters = self.input.iter().map(|param| param.as_python()).collect::<Vec<String>>().join(", ");
        format!("def {}({}) -> {}: \n    pass\n", self.name, parameters, self.output.as_python())
    }
    fn as_javascript(&self) -> String {
        format!("{}", self.name)
    }
}


impl FunctionSignature {
    pub fn list_dependencies(&self) -> Vec<&Dependency> {
        // return a list of all dependencies
        self.input.iter().flat_map(|param| {
            match &param.ptype {
                ParameterType::Dependency(dep) => Some(dep),
                _ => None,
            }
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_python_test() {
        let ast = FunctionSignature { 
            name: String::from("get_users_for_account"), 
            input: vec![
                FunctionParameter { 
                    name: String::from("http_client"), 
                    ptype: ParameterType::Dependency(
                        Dependency { dependency_name: String::from("requests") }
                    )
                },
                FunctionParameter {
                    name: String::from("account_ids"), 
                    ptype: ParameterType::ApplicationParentType(
                        ApplicationParentType {
                            type_name: String::from("List"),
                            children: vec![
                                ParameterType::ApplicationType(
                                    ApplicationType { type_name: String::from("AccountId") }
                                )
                            ]
                        }
                    )
                }
            ], 
            output: ParameterType::ApplicationParentType(
                ApplicationParentType {
                    type_name: String::from("Result"),
                    children: vec![
                        ParameterType::ApplicationParentType(
                            ApplicationParentType {
                                type_name: String::from("List"),
                                children: vec![
                                    ParameterType::ApplicationType(
                                        ApplicationType { type_name: String::from("User") }
                                    )
                                ]
                            }
                        ),
                        ParameterType::ApplicationType(
                            ApplicationType { type_name: String::from("ErrorMsg") }
                        )
                    ]
                }
            )
        };
        let result = ast.as_python();
        let expected = r#"def get_users_for_account(http_client: requests, account_ids: List[AccountId]) -> Result[List[User], ErrorMsg]:"#;
        assert_eq!(result, expected)
    }
}
