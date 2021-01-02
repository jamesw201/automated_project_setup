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

&nbsp;

## Tasks
[√] generate application files from function signature ASTs and mustache templates  
[√] select mocks for dependencies  
[√] create directories with __init__.py or mod.rs files  
[ ] solve how do we get default data from types?  
[ ] generate data from default implementations of types  
[ ] generate tests files  
[ ] generate the main/index file with partially applied dependencies  
[ ] generate default objects from their data types  

&nbsp;

When creating an application the error states are mainly driven by the third party async integrations. Ie. if you import 
an http client then you'll always need to accomodate the possibility of 400/500 errors. If you import the aws dynamo sdk 
then you'll need to accomodate the possibility of something like inadeqate capacity provision errors. So if we know this and we have the basic function signatures of our application then we should be able to generate unit tests which force us to appropriately handle all potential bad outcomes.

This stops us reinventing the wheel every time we programme a workflow that includes async integrations. We can save hours of development time and build with confidence knowing that our software behaves appropriately when the inevitable fault scenarios occur.


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

### AI Ideas
the User writes a brief comment for each function. The AI uses the comment to help convert between the input type and the output type.

- We could employ a transform like config approach for the unimplemented methods to provide the functionality. Although in this case it would need to work with algebraic datatypes and generate functional code.

Even if this can't be used in conjunction with AI from the start, it could generate the data needed to inform an AI over time.

- it would be nice to describe a method's functionality by speech and have that transcribed to text which is fed into our deep learning model which converts it to code in real time.

Ie. for a given function:
```
- def get_active_users(records: List[User]) -> Result[ActiveUsers, ErrorMsg]
```
The developer would describe the functionality as "filters on records, returning those that have the 'active' flag set to true as ActiveUsers".

In python this would generate:
```
def get_active_users(records: List[User]) -> Result[ActiveUsers, ErrorMsg]:
    return [ActiveUser(user) for user in records if user.active == true]
```

To achieve this we'd need a good python->english parser. Given a predictable parser that at least handles simple functional transformations well, we could generate large volumes of training data.

Users would provide less text but with the aid of an existing English AI model we could transfer learn well between the User's somewhat inaccurate function description and one which we can use to drive code generation.

#### Idea
Parse python into english sentences (might be easier if it's predominently functional map/filter/reduce). Use that along with the function signature AST. Employ transfer learning by taking a model that already handles english language well and train it on the code->english sentences. See if that combination can give us english->code capabilities...
