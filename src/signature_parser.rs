use serde::{Deserialize, Serialize};
extern crate nom;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_until},
    character::complete::{alphanumeric1, char, multispace0, space0},
    multi::separated_list0,
    sequence::{preceded, separated_pair, terminated},
    IResult
};


#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ApplicationType {
    type_name: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ApplicationParentType {
    type_name: String,
    children: Vec<ParameterType>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Dependency {
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
    output: ParameterType,
}


fn valid_type_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '[' || c == ']' || c == '{' || c == '}'
}


fn valid_type_identifier(i: &str) -> IResult<&str, &str> {
    take_while(valid_type_identifier_char)(i)
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


fn parse_application_type(i: &str) -> IResult<&str, ParameterType> {
    let (rest, result) = alphanumeric1(i)?;
    Ok((
        rest, ParameterType::ApplicationType( ApplicationType { type_name: String::from(result) } )
    ))
}


fn parse_parent_type(i: &str) -> IResult<&str, ParameterType> {
    let (bracket_rest, type_name) = alphanumeric1(i)?;
    
    let (rest, bracket_contents) = preceded(tag("["), terminated(separated_list0(tag(","), parse_type), tag("]")))(bracket_rest)?;

    let param = ParameterType::ApplicationParentType(
        ApplicationParentType { type_name: String::from(type_name), children: bracket_contents }
    );

    Ok((rest, param))
}


fn parse_dependency_type(i: &str) -> IResult<&str, ParameterType> {
    let (rest, result) = preceded(preceded(space0, tag("{")), terminated(alphanumeric1, tag("}")))(i)?;

    let param = ParameterType::Dependency ( Dependency{dependency_name: String::from(result) });
    Ok((rest, param))
}


fn parse_type(i: &str) -> IResult<&str, ParameterType> {
    preceded(
        space0,
        alt((
            parse_dependency_type,
            parse_parent_type,
            parse_application_type,
        ))
    )(i)
}


fn parse_argument(i: &str) -> IResult<&str, FunctionParameter> {
    let (rest, (left, right)) = separated_pair(preceded(space0, valid_identifier), preceded(space0, char(':')), preceded(space0, valid_type_identifier))(i)?;
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

fn parse_output(i: &str) -> IResult<&str, ParameterType> {
    preceded(
        preceded(space0, tag("->")), 
            preceded(space0, parse_type)
    )(i)
}

    
#[allow(dead_code)]
pub fn root(i: &str) -> IResult<&str, FunctionSignature> {
    let (rest, function_name) = parse_function_name(i)?;
    let (rest, params) = parse_function_arguments(rest)?;
    let (_, output) = parse_output(rest)?;

    let function_signature = FunctionSignature {
        name: String::from(function_name),
        input: params,
        output,
    };

    Ok(("", function_signature))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn function_signature_test() {
        let data = r#"get_users_for_account(http_client: {requests}, account_ids: List[AccountId]) -> Result[List[User], ErrorMsg]"#;
        let expected = FunctionSignature { 
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
    fn parse_nested_parenttype_test() {
        let data = "AccountId[User[Email]]";
        let result = parse_type(data);
        let expected = ParameterType::ApplicationParentType(
            ApplicationParentType { 
                type_name: String::from("AccountId"),
                children: vec![
                    ParameterType::ApplicationParentType(
                        ApplicationParentType { 
                            type_name: String::from("User"),
                            children: vec![
                                ParameterType::ApplicationType(
                                    ApplicationType { type_name: String::from("Email") }
                                )
                            ]
                        }
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
    // fn invalid_syntax_function_signature_test() {
    // }
}
