use super::transformer::{TransformResult, Transformer};
use crate::transformer::Globals;
use serde::{Deserialize, Serialize};

pub mod none;
pub use none::NoneTransformer;

mod internet;
pub use internet::{EmailKind, EmailTransformer, IpTransformer, PasswordTransformer};

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

mod fk;
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
    ("words", Words, WordsTransformer),
    ("first_name", FirstName, FirstNameTransformer),
    ("last_name", LastName, LastNameTransformer),
    ("city", City, CityTransformer),
    ("phone", Phone, PhoneTransformer),
    ("pipeline", Pipeline, PipelineTransformer<Transformers>),
    ("capitalize", Capitalize, CapitalizeTransformer),
    ("template", Template, TemplateTransformer),
    ("digit", Digit, DigitTransformer),
    ("random_num", RandomNum, RandomNumberTransformer),
    ("password", Password, PasswordTransformer),
    ("datetime", DateTime, RandomDateTimeTransformer),

    ("fk_city_prefix", FkCityPrefix, FkCityPrefixTransformer),
    ("fk_city_suffix", FkCitySuffix, FkCitySuffixTransformer),
    ("fk_city_name", FkCityName, FkCityNameTransformer),
    ("fk_country_name", FkCountryName, FkCountryNameTransformer),
    ("fk_country_code", FkCountryCode, FkCountryCodeTransformer),
    ("fk_street_suffix", FkStreetSuffix, FkStreetSuffixTransformer),
    ("fk_street_name", FkStreetName, FkStreetNameTransformer),
    ("fk_time_zone", FkTimeZone, FkTimeZoneTransformer),
    ("fk_state_name", FkStateName, FkStateNameTransformer),
    ("fk_state_abbr", FkStateAbbr, FkStateAbbrTransformer),
    ("fk_secondary_address_type", FkSecondaryAddressType, FkSecondaryAddressTypeTransformer),
    ("fk_secondary_address", FkSecondaryAddress, FkSecondaryAddressTransformer),
    ("fk_zip_code", FkZipCode, FkZipCodeTransformer),
    ("fk_post_code", FkPostCode, FkPostCodeTransformer),
    ("fk_building_number", FkBuildingNumber, FkBuildingNumberTransformer),
    ("fk_latitude", FkLatitude, FkLatitudeTransformer),
    ("fk_longitude", FkLongitude, FkLongitudeTransformer),

    ("fk_boolean", FkBoolean, FkBooleanTransformer),

    ("fk_date", FkDate, FkDateTransformer),
    ("fk_date_time", FkDateTime, FkDateTimeTransformer),

    ("fk_company_suffix", FkCompanySuffix, FkCompanySuffixTransformer),
    ("fk_company_name", FkCompanyName, FkCompanyNameTransformer),
    ("fk_buzzword", FkBuzzword, FkBuzzwordTransformer),
    ("fk_buzzword_middle", FkBuzzwordMiddle, FkBuzzwordMiddleTransformer),
    ("fk_buzzword_tail", FkBuzzwordTail, FkBuzzwordTailTransformer),
    ("fk_catch_phase", FkCatchPhase, FkCatchPhaseTransformer),
    ("fk_bs_verb", FkBsVerb, FkBsVerbTransformer),
    ("fk_bs_adj", FkBsAdj, FkBsAdjTransformer),
    ("fk_bs_noun", FkBsNoun, FkBsNounTransformer),
    ("fk_bs", FkBs, FkBsTransformer),
    ("fk_profession", FkProfession, FkProfessionTransformer),
    ("fk_industry", FkIndustry, FkIndustryTransformer),

    ("fk_free_email_provider", FkFreeEmailProvider, FkFreeEmailProviderTransformer),
    ("fk_domain_suffix", FkDomainSuffix, FkDomainSuffixTransformer),
    ("fk_free_email", FkFreeEmail, FkFreeEmailTransformer),
    ("fk_safe_email", FkSafeEmail, FkSafeEmailTransformer),
    ("fk_username", FkUsername, FkUsernameTransformer),
    ("fk_password", FkPassword, FkPasswordTransformer),
    ("fk_ipv4", FkIPv4, FkIPv4Transformer),
    ("fk_ipv6", FkIPv6, FkIPv6Transformer),
    ("fk_mac_address", FkMACAddress, FkMACAddressTransformer),
    ("fk_color", FkColor, FkColorTransformer),
    ("fk_user_agent", FkUserAgent, FkUserAgentTransformer),
    ("fk_job_seniority", FkJobSeniority, FkJobSeniorityTransformer),
    ("fk_job_field", FkJobField, FkJobFieldTransformer),
    ("fk_job_position", FkJobPosition, FkJobPositionTransformer),
    ("fk_job_title", FkJobTitle, FkJobTitleTransformer),

    ("fk_word", FkWord, FkWordTransformer),
    ("fk_words", FkWords, FkWordsTransformer),
    ("fk_sentence", FkSentence, FkSentenceTransformer),
    ("fk_sentences", FkSentences, FkSentencesTransformer),
    ("fk_paragraph", FkParagraph, FkParagraphTransformer),
    ("fk_paragraphs", FkParagraphs, FkParagraphsTransformer),

    ("fk_first_name", FkFirstName, FkFirstNameTransformer),
    ("fk_last_name", FkLastName, FkLastNameTransformer),
    ("fk_name_suffix", FkNameSuffix, FkNameSuffixTransformer),
    ("fk_person_title", FkPersonTitle, FkPersonTitleTransformer),
    ("fk_person_name", FkPersonName, FkPersonNameTransformer),
    ("fk_person_name_with_title", FkPersonNameWithTitle, FkPersonNameWithTitleTransformer),

    ("fk_digit", FkDigit, FkDigitTransformer),

    ("fk_phone_number", FkPhoneNumber, FkPhoneNumberTransformer),
    ("fk_cell_number", FkCellNumber, FkCellNumberTransformer),

    ("fk_file_path", FkFilePath, FkFilePathTransformer),
    ("fk_file_name", FkFileName, FkFileNameTransformer),
    ("fk_file_extension", FkFileExtension, FkFileExtensionTransformer),
    ("fk_dir_path", FkDirPath, FkDirPathTransformer),

    ("fk_currency_code", FkCurrencyCode, FkCurrencyCodeTransformer),
    ("fk_currency_name", FkCurrencyName, FkCurrencyNameTransformer),
    ("fk_currency_symbol", FkCurrencySymbol, FkCurrencySymbolTransformer)
];
