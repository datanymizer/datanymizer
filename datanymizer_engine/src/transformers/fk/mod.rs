//! We use macros to easily create transformers for the fakers from
//! [fake](https://github.com/cksac/fake-rs) crate.
//! To create a new faker-based transformer please refer to
//! [docs/new_fk_transformer.md](/datanymizer_engine/docs/new_fk_transformer.md).

pub mod sql_value;

use crate::{
    locale::{LocaleConfig, Localized, LocalizedFaker},
    transformer::{TransformContext, TransformResult, TransformerDefaults},
    Transformer,
};
use fake::{
    faker::{
        address::raw::*,
        boolean::raw::*,
        chrono::raw::*,
        company::raw::*,
        currency::raw::*,
        filesystem::raw::*,
        internet::raw::*,
        job::raw::{
            Field as JobField, Position as JobPosition, Seniority as JobSeniority,
            Title as JobTitle,
        },
        lorem::raw::*,
        name::raw::{
            FirstName, LastName, Name as PersonName, NameWithTitle as PersonNameWithTitle,
            Suffix as NameSuffix, Title as PersonTitle,
        },
        number::raw::Digit,
        phone_number::raw::*,
    },
    Fake,
};
use serde::{Deserialize, Serialize};
use sql_value::*;

pub trait FkTransformer<V>: LocalizedFaker<V>
where
    V: AsSqlValue,
{
    fn transform_with_faker(&self) -> TransformResult {
        Ok(Some(V::sql_value(self.localized_fake())))
    }

    fn set_defaults_for_faker(&mut self, defaults: &TransformerDefaults) {
        if self.locale().is_none() {
            self.set_locale(Some(defaults.locale));
        }
    }
}

/// This macro defines a document test for a faker-based transformer.
/// `$ser` - serialization name that also used in [Transformers] enum (e.g., `city`).
/// `$tr` - transformer identifier (e.g., `CityTransformer`).
macro_rules! fk_doctest {
    ( $ser:literal, $tr:ident ) => {
        concat!(
            "# use datanymizer_engine::{\n",
            "#   Transformer, Transformers, transformers::",
            stringify!($tr),
            "\n",
            "# };\n",
            "let cfg = \"",
            $ser,
            ": {}\";\n",
            "let t: Transformers = serde_yaml::from_str(cfg).unwrap();\n",
            "let s = t.transform(\"table.field\", \"t\", &None).unwrap().unwrap();\n",
            "println!(\"{}\", s);\n",
            "# assert!(s.len() > 0);\n"
        )
    };
}

/// This macro defines a configuration doc section test for a faker-based transformers
/// depends on configuration signature.
macro_rules! fk_config_example {
    ( Empty ) => {
        ""
    };

    ( Ratio ) => {
        concat!(
            "      # The probability of TRUE value\n",
            "      ratio: 50\n"
        )
    };

    ( Count ) => {
        concat!(
            "      # Min count\n",
            "      min: 2\n",
            "      # Max count\n",
            "      max: 5\n"
        )
    };
}

/// This macro defines an entire document comment for a faker-based transformer.
/// `$desc` - transformer description.
/// `$ser` - serialization name that also used in [Transformers] enum (e.g., `city`).
/// `$tr` - transformer identifier (e.g., `CityTransformer`).
/// `$cfg` - transformer configuration signature (e.g., `Empty`).
macro_rules! fk_doc_comment {
    ( $desc:literal, $ser:literal, $tr:ident, $cfg:ident ) => {
        concat!(
            $desc,
            "\n\n",
            "# Config example:\n",
            "The default:\n",
            "```yaml\n",
            "rules:\n",
            "  field_name:\n",
            "    ",
            $ser,
            ": {}\n",
            "```\n",
            "This is equal to:\n",
            "```yaml\n",
            "rules:\n",
            "  field_name:\n",
            "    ",
            $ser,
            ":\n",
            "      locale: EN\n\n",
            fk_config_example!($cfg),
            "```\n\n",
            "# Example:\n",
            "```rust\n",
            fk_doctest!($ser, $tr),
            "```"
        )
    };
}

/// This macro defines a faker-based transformer struct itself and also implements [Default] trait
/// for it.
/// `$tr` - transformer identifier (e.g., `CityTransformer`).
/// `$doc` - doc comment contents.
macro_rules! define_fk_struct {
    ( $tr:ident, Empty, $doc:expr ) => {
        #[doc = $doc]
        #[derive(Default, Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
        #[serde(default)]
        pub struct $tr {
            pub locale: Option<LocaleConfig>,
        }
    };

    ( $tr:ident, Ratio, $doc:expr ) => {
        #[doc = $doc]
        #[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
        #[serde(default)]
        pub struct $tr {
            pub locale: Option<LocaleConfig>,
            pub ratio: u8,
        }

        impl Default for $tr {
            fn default() -> Self {
                Self {
                    locale: None,
                    ratio: 50,
                }
            }
        }
    };

    ( $tr:ident, Count, $doc:expr ) => {
        #[doc = $doc]
        #[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
        #[serde(default)]
        pub struct $tr {
            pub locale: Option<LocaleConfig>,
            pub min: usize,
            pub max: usize,
        }

        impl Default for $tr {
            fn default() -> Self {
                Self {
                    locale: None,
                    min: 2,
                    max: 5,
                }
            }
        }
    };
}

/// This macro implements [LocalizedFaker] trait for a transformer.
/// `$fk` - faker type (e.g. [CityName]).
/// `$sql` - faker value type (must implements [AsSqlValue] trait).
macro_rules! impl_localized_faker {
    ( $fk:ident, $sql:ty, Empty ) => {
        fn fake<L: Copy + fake::locales::Data>(&self, l: L) -> $sql {
            $fk(l).fake()
        }
    };

    ( $fk:ident, $sql:ty, Ratio ) => {
        fn fake<L: Copy + fake::locales::Data>(&self, l: L) -> $sql {
            $fk(l, self.ratio).fake()
        }
    };

    ( $fk:ident, $sql:ty, Count ) => {
        fn fake<L: Copy + fake::locales::Data>(&self, l: L) -> $sql {
            $fk(l, self.min..self.max + 1).fake()
        }
    };
}

/// This macro defines a single faker-based transformer.
/// `$desc` - transformer description.
/// `$ser` - serialization name that also used in [Transformers] enum (e.g., `city`).
/// `$tr` - transformer identifier (e.g., `CityTransformer`).
/// `$fk` - faker type (e.g. [CityName]).
/// `$sql` - faker value type (must implements [AsSqlValue] trait).
/// `$cfg` - transformer configuration signature (e.g., `Empty`).
macro_rules! define_fk_transformer {
    ( $desc:literal, ($ser:literal, $tr:ident, $fk:ident, $sql:ty, $cfg:ident) ) => {
        define_fk_struct! { $tr, $cfg, fk_doc_comment!($desc, $ser, $tr, $cfg) }

        impl Localized for $tr {
            fn locale(&self) -> Option<LocaleConfig> {
                self.locale
            }

            fn set_locale(&mut self, l: Option<LocaleConfig>) {
                self.locale = l;
            }
        }

        impl LocalizedFaker<$sql> for $tr {
            impl_localized_faker!($fk, $sql, $cfg);
        }

        impl FkTransformer<$sql> for $tr {}

        impl Transformer for $tr {
            fn transform(
                &self,
                _field_name: &str,
                _field_value: &str,
                _ctx: &Option<TransformContext>,
            ) -> TransformResult {
                self.transform_with_faker()
            }

            fn set_defaults(&mut self, defaults: &TransformerDefaults) {
                self.set_defaults_for_faker(defaults);
            }
        }
    };
}

/// This macro defines faker-based transformers.
macro_rules! define_fk_transformers {
    ( $(  $desc:literal, $tt:tt ),* ) => {
        $(
            define_fk_transformer!($desc, $tt);
        )*
    };
}

define_fk_transformers![
    "Gets a city name.",
    ("city", CityTransformer, CityName, String, Empty),
    "Gets a city prefix (e.g., `North`- or `East`-).",
    ("city_prefix", CityPrefixTransformer, CityPrefix, String, Empty),
    "Gets a city suffix (e.g., -`town`, -`berg` or -`ville`).",
    ("city_suffix", CitySuffixTransformer, CitySuffix, String, Empty),
    "Gets a country name.",
    ("country_name", CountryNameTransformer, CountryName, String, Empty),
    "Gets a country code (e.g., `RU`).",
    ("country_code", CountryCodeTransformer, CountryCode, String, Empty),
    "Gets a street suffix (e.g., `Avenue` or `Highway`).",
    ("street_suffix", StreetSuffixTransformer, StreetSuffix, String, Empty),
    "Gets a street name.",
    ("street_name", StreetNameTransformer, StreetName, String, Empty),
    "Gets a street suffix.",
    ("time_zone", TimeZoneTransformer, TimeZone, String, Empty),
    "Gets a time zone (e.g., `Europe/London`).",
    ("state_name", StateNameTransformer, StateName, String, Empty),
    "Gets a state (or the equivalent) abbreviation (e.g., `AZ` or `LA`).",
    ("state_abbr", StateAbbrTransformer, StateAbbr, String, Empty),
    "Gets a dwelling unit type (e.g., `Apt.` or `Suit.`).",
    ("dwelling_type", DwellingTypeTransformer, SecondaryAddressType, String, Empty),
    "Gets a dwelling unit part of the address (apartment, flat...).",
    ("dwelling", DwellingTransformer, SecondaryAddress, String, Empty),
    "Gets a zip code.",
    ("zip_code", ZipCodeTransformer, ZipCode, String, Empty),
    "Gets a post code.",
    ("post_code", PostCodeTransformer, PostCode, String, Empty),
    "Gets a building number.",
    ("building_number", BuildingNumberTransformer, BuildingNumber, String, Empty),
    "Gets a latitude.",
    ("latitude", LatitudeTransformer, Latitude, GenericFloat, Empty),
    "Gets a longitude.",
    ("longitude", LongitudeTransformer, Longitude, GenericFloat, Empty),
    "Gets a boolean value (TRUE/FALSE), with a given probability.",
    ("boolean", BooleanTransformer, Boolean, bool, Ratio),
    "Gets a random date (without formatting).",
    ("raw_date", RawDateTransformer, Date, GenericDate, Empty),
    "Gets a random datetime (without formatting).",
    ("raw_datetime", RawDateTimeTransformer, DateTime, GenericDateTime, Empty),
    "Gets a company name suffix (e.g., `Inc.` or `LLC`).",
    ("company_suffix", CompanySuffixTransformer, CompanySuffix, String, Empty),
    "Gets a company name.",
    ("company_name", CompanyNameTransformer, CompanyName, String, Empty),
    "Gets a company motto.",
    ("company_motto", CompanyMottoTransformer, CatchPhase, String, Empty),
    "Gets a head component of a company motto.",
    ("company_motto_head", CompanyMottoHeadTransformer, Buzzword, String, Empty),
    "Gets a middle component of a company motto.",
    ("company_motto_middle", CompanyMottoMiddleTransformer, BuzzwordMiddle, String, Empty),
    "Gets a tail component of a company motto.",
    ("company_motto_tail", CompanyMottoTailTransformer, BuzzwordTail, String, Empty),
    "Gets a company activity description (e.g., `integrate vertical markets`).",
    ("company_activity", CompanyActivityTransformer, Bs, String, Empty),
    "Gets a company activity verb.",
    ("company_activity_verb", CompanyActivityVerbTransformer, BsVerb, String, Empty),
    "Gets a company activity adjective.",
    ("company_activity_adj", CompanyActivityAdjTransformer, BsAdj, String, Empty),
    "Gets a company activity noun.",
    ("company_activity_noun", CompanyActivityNounTransformer, BsNoun, String, Empty),
    "Gets a profession name.",
    ("profession", ProfessionTransformer, Profession, String, Empty),
    "Gets an industry name.",
    ("industry", IndustryTransformer, Industry, String, Empty),
    "Gets a free email provider name (e. g., `gmail.com`)",
    ("free_email_provider", FreeEmailProviderTransformer, FreeEmailProvider, String, Empty),
    "Gets a domain suffix (e.g., `com`).",
    ("domain_suffix", DomainSuffixTransformer, DomainSuffix, String, Empty),
    "Gets an user name (login).",
    ("username", UsernameTransformer, Username, String, Empty),
    "Gets a MAC address.",
    ("mac_address", MACAddressTransformer, MACAddress, String, Empty),
    "Gets a color code (e.g., `#ffffff`).",
    ("color", ColorTransformer, Color, String, Empty),
    "Gets an User-Agent header.",
    ("user_agent", UserAgentTransformer, UserAgent, String, Empty),
    "Gets a job seniority (e.g., `Lead`, `Senior` or `Junior`).",
    ("job_seniority", JobSeniorityTransformer, JobSeniority, String, Empty),
    "Gets a job field.",
    ("job_field", JobFieldTransformer, JobField, String, Empty),
    "Gets a job position.",
    ("job_position", JobPositionTransformer, JobPosition, String, Empty),
    "Gets a job title (seniority + field + position).",
    ("job_title", JobTitleTransformer, JobTitle, String, Empty),
    "Gets a \"lorem\" word.",
    ("word", WordTransformer, Word, String, Empty),
    "Gets several \"lorem\" words (you can specify a count).",
    ("words", WordsTransformer, Words, Vec<String>, Count),
    "Gets a \"lorem\" sentence (you can specify a count of words).",
    ("sentence", SentenceTransformer, Sentence, String, Count),
    "Gets several \"lorem\" sentences (you can specify a count).",
    ("sentences", SentencesTransformer, Sentences, Vec<String>, Count),
    "Gets a \"lorem\" paragraph (you can specify a count sentences).",
    ("paragraph", ParagraphTransformer, Paragraph, String, Count),
    "Gets several \"lorem\" paragraphs (you can specify a count).",
    ("paragraphs", ParagraphsTransformer, Paragraphs, Vec<String>, Count),
    "Gets a person name.",
    ("person_name", PersonNameTransformer, PersonName, String, Empty),
    "Gets the first name",
    ("first_name", FirstNameTransformer, FirstName, String, Empty),
    "Gets the last name",
    ("last_name", LastNameTransformer, LastName, String, Empty),
    "Gets a name suffix (e.g., `Jr.`)",
    ("name_suffix", NameSuffixTransformer, NameSuffix, String, Empty),
    "Gets a person name title (e.g., `Mr` or `Ms`).",
    ("person_title", PersonTitleTransformer, PersonTitle, String, Empty),
    "Gets a person name with title.",
    ("person_name_with_title", PersonNameWithTitleTransformer, PersonNameWithTitle, String, Empty),
    "Gets a digit symbol (e.g., `2` or `5` for the English locale).",
    ("digit", DigitTransformer, Digit, String, Empty),
    "Gets a local phone number (for a given locale).",
    ("local_phone", LocalPhoneTransformer, PhoneNumber, String, Empty),
    "Gets a local cell phone number (for a given locale).",
    ("local_cell_phone", LocalCellPhoneTransformer, CellNumber, String, Empty),
    "Gets a file path.",
    ("file_path", FilePathTransformer, FilePath, String, Empty),
    "Gets a file name.",
    ("file_name", FileNameTransformer, FileName, String, Empty),
    "Gets a file extension.",
    ("file_extension", FileExtensionTransformer, FileExtension, String, Empty),
    "Gets a file directory path",
    ("dir_path", DirPathTransformer, DirPath, String, Empty),
    "Gets a currency code (e.g., `EUR` or `USD`).",
    ("currency_code", CurrencyCodeTransformer, CurrencyCode, String, Empty),
    "Gets a currency name.",
    ("currency_name", CurrencyNameTransformer, CurrencyName, String, Empty),
    "Gets a currency symbol.",
    ("currency_symbol", CurrencySymbolTransformer, CurrencySymbol, String, Empty)
];

#[cfg(test)]
mod tests {
    use super::*;

    mod deserialization {
        use super::*;

        #[test]
        fn city_name() {
            let cfg = "locale: EN";
            let t: CityTransformer = serde_yaml::from_str(cfg).unwrap();
            assert_eq!(
                t,
                CityTransformer {
                    locale: Some(LocaleConfig::EN)
                }
            );
        }

        #[test]
        fn boolean() {
            let cfg = r#"
                locale: EN
                ratio: 30
                "#;
            let t: BooleanTransformer = serde_yaml::from_str(cfg).unwrap();
            assert_eq!(
                t,
                BooleanTransformer {
                    locale: Some(LocaleConfig::EN),
                    ratio: 30
                }
            );
        }

        #[test]
        fn words() {
            let cfg = r#"
                locale: EN
                min: 2
                max: 3
                "#;
            let t: WordsTransformer = serde_yaml::from_str(cfg).unwrap();
            assert_eq!(
                t,
                WordsTransformer {
                    locale: Some(LocaleConfig::EN),
                    min: 2,
                    max: 3
                }
            );
        }

        #[test]
        fn words_default() {
            let cfg = "{}";
            let t: WordsTransformer = serde_yaml::from_str(cfg).unwrap();
            assert_eq!(
                t,
                WordsTransformer {
                    locale: None,
                    min: 2,
                    max: 5
                }
            );
        }

        #[test]
        fn default_locale() {
            let cfg = "{}";
            let t: CityTransformer = serde_yaml::from_str(cfg).unwrap();
            assert_eq!(t, CityTransformer { locale: None });
        }

        #[test]
        fn default_config() {
            let cfg = "locale: RU";
            let t: BooleanTransformer = serde_yaml::from_str(cfg).unwrap();
            assert_eq!(
                t,
                BooleanTransformer {
                    locale: Some(LocaleConfig::RU),
                    ratio: 50
                }
            );
        }

        #[test]
        fn default_all() {
            let cfg = "{}";
            let t: BooleanTransformer = serde_yaml::from_str(cfg).unwrap();
            assert_eq!(t, BooleanTransformer::default());
        }
    }

    mod defaults {
        use super::*;

        #[test]
        fn locale_is_none() {
            let mut t = CityTransformer { locale: None };
            t.set_defaults(&TransformerDefaults {
                locale: LocaleConfig::RU,
            });
            assert_eq!(t.locale, Some(LocaleConfig::RU));
        }

        #[test]
        fn locale_is_some() {
            let mut t = CityTransformer {
                locale: Some(LocaleConfig::EN),
            };
            t.set_defaults(&TransformerDefaults {
                locale: LocaleConfig::RU,
            });
            assert_eq!(t.locale, Some(LocaleConfig::EN));
        }
    }

    #[test]
    fn boolean() {
        let t = BooleanTransformer {
            locale: None,
            ratio: 0,
        };
        assert_eq!(
            t.transform("table.field", "t", &None),
            Ok(Some(String::from("FALSE")))
        );

        let t = BooleanTransformer {
            locale: None,
            ratio: 100,
        };
        assert_eq!(
            t.transform("table.field", "t", &None),
            Ok(Some(String::from("TRUE")))
        );
    }

    #[test]
    fn city() {
        let t = CityTransformer::default();
        let value = t.transform("table.field", "t", &None).unwrap().unwrap();
        assert!(value.len() > 1);
        assert!(('A'..='Z').contains(&value.chars().next().unwrap()));
    }

    #[test]
    fn country_name() {
        let t = CountryNameTransformer::default();
        let value = t.transform("table.field", "t", &None).unwrap().unwrap();
        assert!(value.len() > 1);
        assert!(('A'..='Z').contains(&value.chars().next().unwrap()));
    }

    #[test]
    fn first_name() {
        let t = FirstNameTransformer::default();
        let value = t.transform("table.field", "t", &None).unwrap().unwrap();
        assert!(value.len() > 1);
        assert!(('A'..='Z').contains(&value.chars().next().unwrap()));
    }

    #[test]
    fn last_name() {
        let t = LastNameTransformer::default();
        let value = t.transform("table.field", "t", &None).unwrap().unwrap();
        assert!(value.len() > 1);
        assert!(('A'..='Z').contains(&value.chars().next().unwrap()));
    }

    #[test]
    fn longitude() {
        let t = LongitudeTransformer::default();
        let value = t.transform("table.field", "t", &None).unwrap().unwrap();
        let value: f32 = value.parse().unwrap();
        assert!(value < 360.0 && value > -360.0);
    }

    #[test]
    fn datetime() {
        let t = RawDateTimeTransformer::default();
        let value = t.transform("table.field", "t", &None).unwrap().unwrap();
        // yyyy-mm-dd hh:mm:ss
        assert_eq!(value.len(), 19);
    }

    #[test]
    fn words() {
        let t = WordsTransformer {
            locale: None,
            min: 5,
            max: 5,
        };
        let value = t.transform("table.field", "t", &None).unwrap().unwrap();
        assert_eq!(value.split(" ").count(), 5);
    }

    #[test]
    fn zh_tw_locale() {
        let t = PersonNameTransformer {
            locale: Some(LocaleConfig::ZH_TW),
        };
        let value = t.transform("table.field", "t", &None).unwrap().unwrap();
        assert!(!('A'..='Z').contains(&value.chars().next().unwrap()));
    }
}
