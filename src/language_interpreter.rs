use crate::domains::schema::MockConfig;

pub trait LanguageInterpreter {
    fn as_python(&self) -> String;
    fn as_javascript(&self) -> String;
}

pub trait LanguageInterpreterForUnitTest {
    fn as_python(&self, mock_ref: &str, mock_config: &MockConfig, target_ref: &str) -> String;
    fn as_javascript(&self, mock_ref: &str, mock_config: &MockConfig, target_ref: &str) -> String;
}