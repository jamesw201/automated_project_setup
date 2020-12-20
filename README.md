---
tags: project idea
---
# Automated project setup
 
- create types / select types from previous projects
- create a schema of function signatures for your workflow
- generate code...
- fill in the function implementations

Inputs to the programme can be converted into algebraic types using the  input_parser application.
<br />


A **basic** schema should be enough to setup a programme using functional patterns which ensure that best practice is followed for the following critical application steps:

- validation and parsing into data classes
- clear and direct application flow/structure
- error handling
- small and clear tests

There will be no magic or hidden code, just simple design decisions leading to a dependable skeleton application. From which a developer only needs to fill in the function logic.

The challenge is the same in every language so it should be possible to handle it with a single mechanism.

Generating all of the files and tests up front will leave more time for thinking about edge cases and going to the pub.

If there is an error in the way the Developer has specified their types then the application will let you give you a detailed description why.

### It can also
- generate documentation via the most common language docs approach.
- generate component tests where it sees apis endpoints are being developed.


### Core components
- a library which converts input into algebraic data types (a different implementation will be needed for each language).
- a monadic library which allows to return ```Result[success, failure]``` types
- a directory of common mocks. The application will pull any appropriate mocks into your application test directory on generation.

### Schema
```
project-name: 'my_project'
root-directory: '~/repos/my-repo/'
language: 'javascript/python/rust/go'
dependencies: 
    - requests
      aws-sdk/dynamodb
      aws-sdk/s3
system-types: 'path/to/system-types/directory'
domain-types: 'path/to/domain-types/directory'
files:
    - path: 'src/domains/account'
      methods:
        - get_users_for_account(http_client: {requests}, List[AccountId]) -> Result[List[User], ErrorMsg]
        - get_account(id: AccountId) -> Result[Account, ErrorMsg]
    - path: 'src/domains/user'
      methods:
        - update_user(user_id: UserId) -> Result[None, ErrorMsg]
    - path: 'src/domains/role'
      methods: 
    - path: 'src/cleanup_account_users'
      methods: 
workflow:
    - validate_input
      get_users_for_account
      update_users
      convert_to_dto
      persist_users
```

### Automated test mocks
Dependencies will be referenced in function signatures. This will allow the tool to know how to partially apply them from the top of the app. It will also setup tests at the service level and module level with the appropriate test. Ie. An http client will need to test for:
- 400 response
- 500 response
- 200 response with empty data

DynamoDb (or any other sdk call) dependencies will generate different mocks and tests etc.

### Questions
- how do we handle apis? Probably just add a field called endpoint which looks like: 'GET /domain/api/endpoint{variable}'. This could lead to auto generated component_tests.
- how generic can this be made? How much effort would it take to change the look/feel/structure/design-methodology if someone has different preferences/requirements.
- if a function receives a ```Result[sucess,failure]``` type in its input then we can bind in one way, or if it takes a domain type then we can just bind the success part to the next function in the workflow.

<br />
&nbsp;

## Future goals
[operation replace all colleagues]  
Write ai to fill in the function bodies. It should:
- convert from one type to another
    starting singluar input->to singluar->output
- guess at the correct method call to make on a dependency, this will be aided by the function name which calls the dependency.
    Ie. get_all_users() will most likely call requests.get() 
        update_account() will most likely call something like dynamo.update()

Leverage any existing ai codebase that may be able to do this already...