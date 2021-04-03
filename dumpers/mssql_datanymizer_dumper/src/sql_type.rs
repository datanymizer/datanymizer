#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MsSqlType {
    name: String,
    supported: bool,
}

const SUPPORTED_TYPES: [&str; 28] = [
    "bigint",
    "binary",
    "bit",
    "char",
    "date",
    "datetime",
    "datetime2",
    "datetimeoffset",
    "decimal",
    "float",
    "image",
    "int",
    "money",
    "nchar",
    "ntext",
    "numeric",
    "nvarchar",
    "real",
    "smalldatetime",
    "smallint",
    "smallmoney",
    "text",
    "time",
    "tinyint",
    "uniqueidentifier",
    "varbinary",
    "varchar",
    "xml",
];

impl MsSqlType {
    pub fn new(name: String) -> Self {
        let supported = SUPPORTED_TYPES.contains(&name.as_str());
        Self { name, supported }
    }

    pub fn is_supported(&self) -> bool {
        self.supported
    }
}
