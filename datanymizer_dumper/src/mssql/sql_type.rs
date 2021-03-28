#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MsSqlType(pub String);

const UNSUPPORTED_TYPES: [&str; 2] = ["hierarchyid", "geography"];

impl MsSqlType {
    pub fn has_supported_type(&self) -> bool {
        !UNSUPPORTED_TYPES.contains(&self.0.as_str())
    }
}
