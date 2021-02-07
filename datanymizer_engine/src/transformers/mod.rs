use super::transformer::{TransformResult, Transformer};
use crate::transformer::Globals;
use serde::{Deserialize, Serialize};

pub mod none;
pub use none::NoneTransformer;

mod internet;
pub use internet::{EmailKind, EmailTransformer, IpTransformer, PasswordTransformer};

mod phone;
pub use phone::PhoneTransformer;

mod pipeline;
pub use pipeline::PipelineTransformer;

mod capitalize;
pub use capitalize::CapitalizeTransformer;

mod template;
pub use template::TemplateTransformer;

mod number;
pub use number::RandomNumberTransformer;

mod datetime;
pub use datetime::RandomDateTimeTransformer;

mod fk;
pub use fk::sql_value::AsSqlValue;
pub use fk::*;

macro_rules! define_transformers_enum {
    ( $( ( $ser:literal, $var:ident, $tr:ty ) ),* ) => {
        #[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
        pub enum Transformers {
            $(
                #[serde(rename = $ser)]
                $var($tr),
            )*
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
                    $(
                        $var(ref t) => t.transform(field_name, field_value, globals),
                    )*
                }
            }
        }
    };
}

define_transformers_enum![
    ("none", None, NoneTransformer),
    ("email", Email, EmailTransformer),
    ("ip", IP, IpTransformer),
    ("phone", Phone, PhoneTransformer),
    ("pipeline", Pipeline, PipelineTransformer<Transformers>),
    ("capitalize", Capitalize, CapitalizeTransformer),
    ("template", Template, TemplateTransformer),
    ("random_num", RandomNum, RandomNumberTransformer),
    ("password", Password, PasswordTransformer),
    ("datetime", DateTime, RandomDateTimeTransformer),

    ("city", City, CityTransformer),
    ("city_prefix", CityPrefix, CityPrefixTransformer),
    ("city_suffix", CitySuffix, CitySuffixTransformer),
    ("country_name", CountryName, CountryNameTransformer),
    ("country_code", CountryCode, CountryCodeTransformer),
    ("street_suffix", StreetSuffix, StreetSuffixTransformer),
    ("street_name", StreetName, StreetNameTransformer),
    ("time_zone", TimeZone, TimeZoneTransformer),
    ("state_name", StateName, StateNameTransformer),
    ("state_abbr", StateAbbr, StateAbbrTransformer),
    ("dwelling_type", DwellingType, DwellingTypeTransformer),
    ("dwelling", Dwelling, DwellingTransformer),
    ("zip_code", ZipCode, ZipCodeTransformer),
    ("post_code", PostCode, PostCodeTransformer),
    ("building_number", BuildingNumber, BuildingNumberTransformer),
    ("latitude", Latitude, LatitudeTransformer),
    ("longitude", Longitude, LongitudeTransformer),

    ("boolean", Boolean, BooleanTransformer),

    ("raw_date", RawDate, RawDateTransformer),
    ("raw_datetime", RawDateTime, RawDateTimeTransformer),

    ("company_suffix", CompanySuffix, CompanySuffixTransformer),
    ("company_name", CompanyName, CompanyNameTransformer),
    ("company_motto", CompanyMotto, CompanyMottoTransformer),
    ("company_motto_head", CompanyMottoHead, CompanyMottoHeadTransformer),
    ("company_motto_middle", BCompanyMottoMiddle, CompanyMottoMiddleTransformer),
    ("company_motto_tail", CompanyMottoTail, CompanyMottoTailTransformer),
    ("company_activity", CompanyActivity, CompanyActivityTransformer),
    ("company_activity_verb", CompanyActivityVerb, CompanyActivityVerbTransformer),
    ("company_activity_adj", CompanyActivityAdj, CompanyActivityAdjTransformer),
    ("company_activity_noun", CompanyActivityNoun, CompanyActivityNounTransformer),
    ("profession", Profession, ProfessionTransformer),
    ("industry", Industry, IndustryTransformer),

    ("free_email_provider", FreeEmailProvider, FreeEmailProviderTransformer),
    ("domain_suffix", DomainSuffix, DomainSuffixTransformer),
    ("username", Username, UsernameTransformer),
    ("mac_address", MACAddress, MACAddressTransformer),
    ("color", Color, ColorTransformer),
    ("user_agent", UserAgent, UserAgentTransformer),
    ("job_seniority", JobSeniority, JobSeniorityTransformer),
    ("job_field", JobField, JobFieldTransformer),
    ("job_position", JobPosition, JobPositionTransformer),
    ("job_title", JobTitle, JobTitleTransformer),

    ("word", Word, WordTransformer),
    ("words", Words, WordsTransformer),
    ("sentence", Sentence, SentenceTransformer),
    ("sentences", Sentences, SentencesTransformer),
    ("paragraph", Paragraph, ParagraphTransformer),
    ("paragraphs", Paragraphs, ParagraphsTransformer),

    ("first_name", FirstName, FirstNameTransformer),
    ("last_name", LastName, LastNameTransformer),
    ("name_suffix", NameSuffix, NameSuffixTransformer),
    ("person_title", PersonTitle, PersonTitleTransformer),
    ("person_name", PersonName, PersonNameTransformer),
    ("person_name_with_title", PersonNameWithTitle, PersonNameWithTitleTransformer),

    ("digit", Digit, DigitTransformer),

    ("local_phone", LocalPhone, LocalPhoneTransformer),
    ("local_cell_phone", LocalCellPhone, LocalCellPhoneTransformer),

    ("file_path", FilePath, FilePathTransformer),
    ("file_name", FileName, FileNameTransformer),
    ("file_extension", FileExtension, FileExtensionTransformer),
    ("dir_path", DirPath, DirPathTransformer),

    ("currency_code", CurrencyCode, CurrencyCodeTransformer),
    ("currency_name", CurrencyName, CurrencyNameTransformer),
    ("currency_symbol", CurrencySymbol, CurrencySymbolTransformer)
];
