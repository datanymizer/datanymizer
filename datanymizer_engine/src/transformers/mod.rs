use super::transformer::{TransformResult, Transformer};
use crate::transformer::Globals;
use serde::{Deserialize, Serialize};

mod none;
pub use none::NoneTransformer;

mod internet;
pub use internet::{EmailTransformer, IpTransformer, PasswordTransformer};

mod lorem;
pub use lorem::WordsTransformer;

mod names;
pub use names::{FirstNameTransformer, LastNameTransformer};

mod city;
pub use city::CityTransformer;

mod phone;
pub use phone::PhoneTransformer;

mod pipeline;
pub use pipeline::PipelineTransformer;

mod capitalize;
pub use capitalize::CapitalizeTransformer;

mod template;
pub use template::TemplateTransformer;

mod number;
pub use number::{DigitTransformer, RandomNumberTransformer};

mod datetime;
pub use datetime::RandomDateTimeTransformer;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub enum Transformers {
    #[serde(rename = "none")]
    None(NoneTransformer),
    #[serde(rename = "email")]
    Email(EmailTransformer),
    #[serde(rename = "ip")]
    IP(IpTransformer),
    #[serde(rename = "words")]
    Words(WordsTransformer),
    #[serde(rename = "first_name")]
    FirstName(FirstNameTransformer),
    #[serde(rename = "last_name")]
    LastName(LastNameTransformer),
    #[serde(rename = "city")]
    City(CityTransformer),
    #[serde(rename = "phone")]
    Phone(PhoneTransformer),
    #[serde(rename = "pipeline")]
    Pipeline(PipelineTransformer<Transformers>),
    #[serde(rename = "capitalize")]
    Capitalize(CapitalizeTransformer),
    #[serde(rename = "template")]
    Template(TemplateTransformer),
    #[serde(rename = "digit")]
    Digit(DigitTransformer),
    #[serde(rename = "random_num")]
    RandomNum(RandomNumberTransformer),
    #[serde(rename = "password")]
    Password(PasswordTransformer),
    #[serde(rename = "datetime")]
    DateTime(RandomDateTimeTransformer),
}

impl Transformer for Transformers {
    fn transform(
        &self,
        field_name: &str,
        field_value: &str,
        globals: &Option<Globals>,
    ) -> TransformResult {
        use self::Transformers::*;

        match *self {
            None(ref t) => t.transform(field_name, field_value, globals),
            Email(ref t) => t.transform(field_name, field_value, globals),
            IP(ref t) => t.transform(field_name, field_value, globals),
            Words(ref t) => t.transform(field_name, field_value, globals),
            FirstName(ref t) => t.transform(field_name, field_value, globals),
            LastName(ref t) => t.transform(field_name, field_value, globals),
            City(ref t) => t.transform(field_name, field_value, globals),
            Phone(ref t) => t.transform(field_name, field_value, globals),
            Pipeline(ref t) => t.transform(field_name, field_value, globals),
            Capitalize(ref t) => t.transform(field_name, field_value, globals),
            Template(ref t) => t.transform(field_name, field_value, globals),
            Digit(ref t) => t.transform(field_name, field_value, globals),
            RandomNum(ref t) => t.transform(field_name, field_value, globals),
            Password(ref t) => t.transform(field_name, field_value, globals),
            DateTime(ref t) => t.transform(field_name, field_value, globals),
        }
    }
}
