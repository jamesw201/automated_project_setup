use serde::{Deserialize, Serialize};
extern crate nom;
use nom::{
    branch::alt,
    bytes::complete::{escaped, is_not, tag, take, take_while, take_until},
    character::complete::{alphanumeric1, char, one_of, multispace0, newline, not_line_ending, line_ending, space0, space1},
    multi::{many0, many1, separated_list0, fold_many0},
    sequence::{delimited, preceded, pair, separated_pair, terminated},
    IResult
};


#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct ApplicationType {
    type_name: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct ApplicationParentType {
    type_name: String,
    children: Vec<ParameterType>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Dependency {
    dependency_name: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum ParameterType {
    ApplicationType(ApplicationType),
    ApplicationParentType(ApplicationParentType),
    Dependency(Dependency),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct FunctionParameter {
    name: String,
    ptype: ParameterType,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct FunctionSignature {
    name: String,
    input: Vec<FunctionParameter>,
    output: ApplicationType,
}


fn valid_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}


fn valid_identifier(i: &str) -> IResult<&str, &str> {
    take_while(valid_identifier_char)(i)
}


fn parse_function_name(i: &str) -> IResult<&str, &str> {
    take_until("(")(i)
}


fn parse_type(i: &str) -> IResult<&str, ParameterType> {
    println!("parsing type: {}", &i);
    // TODO: 
    // [ ] handle parent types
    // [ ] handle dependency types
    let (rest, result) = alphanumeric1(i)?;
    Ok((
        rest, ParameterType::ApplicationType( ApplicationType { type_name: String::from(result) } )
    ))
}


fn parse_argument(i: &str) -> IResult<&str, FunctionParameter> {
    let (rest, (left, right)) = separated_pair(preceded(space0, valid_identifier), preceded(space0, char(':')), preceded(space0, alphanumeric1))(i)?;
    let (_, application_type) = parse_type(right)?;
    let param = FunctionParameter { name: String::from(left), ptype: application_type };
    Ok((rest, param))
}


fn parse_function_arguments(i: &str) -> IResult<&str, Vec<FunctionParameter>> {
    preceded(tag("("), 
        terminated(
            separated_list0(
                preceded(space0, tag(",")), preceded(multispace0, parse_argument)
            ), 
            tag(")")
        )
    )(i)
}

    
#[allow(dead_code)]
pub fn root(i: &str) -> IResult<&str, FunctionSignature> {
    println!("input: {}", &i);
    let (rest, function_name) = parse_function_name(i)?;
    let (rest, params) = parse_function_arguments(rest)?;

    let function_signature = FunctionSignature {
        name: String::from(function_name),
        input: params,
        output: ApplicationType { type_name: String::from("Result") },
    };

    Ok(("", function_signature))
}

// TODO:
// [√] match the function name
// [√] match function parameters
// [ ] match nested lists of Types
// [ ] match dependency input
// [ ] match output

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn function_signature_test() {
        let data = r#"get_account(id: AccountId, blablob: Bibble) -> Result[Account, ErrorMsg]"#;
        let expected = FunctionSignature { 
            name: String::from("get_account"), 
            input: vec![
                FunctionParameter { 
                    name: String::from("id"), 
                    ptype: ParameterType::ApplicationType(
                        ApplicationType { type_name: String::from("AccountId") }
                    )
                },
                FunctionParameter {
                    name: String::from("blablob"), 
                    ptype: ParameterType::ApplicationType(
                        ApplicationType { type_name: String::from("Bibble") }
                    )
                }
            ], 
            output: ApplicationType { type_name: String::from("Result") }
        };
        let result = root(data);
        assert_eq!(result, Ok(("", expected)))
    }

    #[test]
    fn parse_type_test() {
        let data = "AccountId";
        let result = parse_type(data);
        let expected = ParameterType::ApplicationType(
            ApplicationType { type_name: String::from("AccountId") }
        );
        assert_eq!(result, Ok(("", expected)))
    }
    
    #[test]
    fn parse_parenttype_test() {
        let data = "AccountId[User]";
        let result = parse_type(data);
        let expected = ParameterType::ApplicationParentType(
            ApplicationParentType { 
                type_name: String::from("AccountId"),
                children: vec![
                    ParameterType::ApplicationType(
                        ApplicationType { type_name: String::from("User") }
                    )
                ]
            }
        );
        assert_eq!(result, Ok(("", expected)))
    }
    
    #[test]
    fn parse_dependency_type_test() {
        let data = "{requests}";
        let result = parse_type(data);
        let expected = ParameterType::Dependency(
            Dependency { 
                dependency_name: String::from("requests"),
            }
        );
        assert_eq!(result, Ok(("", expected)))
    }
    
    #[test]
    fn parse_argument_test() {
        let data = "id: AccountId";
        let result = parse_argument(data);
        let application_type = ApplicationType { type_name: String::from("AccountId") };
        let expected = FunctionParameter { name: String::from("id"), ptype: ParameterType::ApplicationType(application_type) };
        assert_eq!(result, Ok(("", expected)))
    }

    // #[test]
    // fn function_signature_with_dependency_test() {
    //     let data = r#""get_users_for_account(http_client: {requests}, account_ids: List[AccountId]) -> Result[List[User], ErrorMsg]""#;
    //     let expected = FunctionSignature{ name: String::from("someVariable"), input: vec![], output: vec![] };
    //     let result = root(data).unwrap();

    //     assert_eq!(result, ("", expected))
    // }

    // #[test]
    // fn invalid_syntax_function_signature_test() {
    // }
}
