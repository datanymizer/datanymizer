mod configs;
mod sql_value;

use crate::{
    locale::{LocaleConfig, Localized, LocalizedFaker},
    transformer::{Globals, TransformResult},
    Transformer,
};
use configs::*;
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

trait FkTransformer<V>: LocalizedFaker<V>
where
    V: AsSqlValue,
{
    fn transform_with_faker(&self) -> TransformResult {
        Ok(Some(V::sql_value(self.localized_fake())))
    }
}

macro_rules! impl_localized_faker {
    ( $fk:ident, $sql:ty, EmptyConfig ) => {
        fn fake<L: Copy + fake::locales::Data>(&self, l: L) -> $sql {
            $fk(l).fake()
        }
    };

    ( $fk:ident, $sql:ty, RatioConfig ) => {
        fn fake<L: Copy + fake::locales::Data>(&self, l: L) -> $sql {
            $fk(l, self.config.ratio).fake()
        }
    };

    ( $fk:ident, $sql:ty, LenConfig ) => {
        fn fake<L: Copy + fake::locales::Data>(&self, l: L) -> $sql {
            $fk(l, self.config.len.range()).fake()
        }
    };

    ( $fk:ident, $sql:ty, CountConfig ) => {
        fn fake<L: Copy + fake::locales::Data>(&self, l: L) -> $sql {
            $fk(l, self.config.count.range()).fake()
        }
    };
}

macro_rules! define_fk_transformer {
    ( ($tr:ident, $fk:ident, $sql:ty, $cfg:ident) ) => {
        #[derive(Default, Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
        #[serde(default)]
        pub struct $tr {
            locale: LocaleConfig,
            config: $cfg,
        }

        impl Localized for $tr {
            fn locale(&self) -> LocaleConfig {
                self.locale
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
                _globals: &Option<Globals>,
            ) -> TransformResult {
                self.transform_with_faker()
            }
        }
    };
}

macro_rules! define_fk_transformers {
    ( $( $tt:tt ),* ) => {
        $(
            define_fk_transformer!($tt);
        )*
    };
}

define_fk_transformers![
    (FkCityPrefixTransformer, CityPrefix, String, EmptyConfig),
    (FkCitySuffixTransformer, CitySuffix, String, EmptyConfig),
    (FkCityNameTransformer, CityName, String, EmptyConfig),
    (FkCountryNameTransformer, CountryName, String, EmptyConfig),
    (FkCountryCodeTransformer, CountryCode, String, EmptyConfig),
    (FkStreetSuffixTransformer, StreetSuffix, String, EmptyConfig),
    (FkStreetNameTransformer, StreetName, String, EmptyConfig),
    (FkTimeZoneTransformer, TimeZone, String, EmptyConfig),
    (FkStateNameTransformer, StateName, String, EmptyConfig),
    (FkStateAbbrTransformer, StateAbbr, String, EmptyConfig),
    (
        FkSecondaryAddressTypeTransformer,
        SecondaryAddressType,
        String,
        EmptyConfig
    ),
    (
        FkSecondaryAddressTransformer,
        SecondaryAddress,
        String,
        EmptyConfig
    ),
    (FkZipCodeTransformer, ZipCode, String, EmptyConfig),
    (FkPostCodeTransformer, PostCode, String, EmptyConfig),
    (
        FkBuildingNumberTransformer,
        BuildingNumber,
        String,
        EmptyConfig
    ),
    (FkLatitudeTransformer, Latitude, GenericFloat, EmptyConfig),
    (FkLongitudeTransformer, Longitude, GenericFloat, EmptyConfig),
    (FkBooleanTransformer, Boolean, bool, RatioConfig),
    (FkDateTransformer, Date, GenericDate, EmptyConfig),
    (
        FkDateTimeTransformer,
        DateTime,
        GenericDateTime,
        EmptyConfig
    ),
    (
        FkCompanySuffixTransformer,
        CompanySuffix,
        String,
        EmptyConfig
    ),
    (FkCompanyNameTransformer, CompanyName, String, EmptyConfig),
    (FkBuzzwordTransformer, Buzzword, String, EmptyConfig),
    (
        FkBuzzwordMiddleTransformer,
        BuzzwordMiddle,
        String,
        EmptyConfig
    ),
    (FkBuzzwordTailTransformer, BuzzwordTail, String, EmptyConfig),
    (FkCatchPhaseTransformer, CatchPhase, String, EmptyConfig),
    (FkBsVerbTransformer, BsVerb, String, EmptyConfig),
    (FkBsAdjTransformer, BsAdj, String, EmptyConfig),
    (FkBsNounTransformer, BsNoun, String, EmptyConfig),
    (FkBsTransformer, Bs, String, EmptyConfig),
    (FkProfessionTransformer, Profession, String, EmptyConfig),
    (FkIndustryTransformer, Industry, String, EmptyConfig),
    (
        FkFreeEmailProviderTransformer,
        FreeEmailProvider,
        String,
        EmptyConfig
    ),
    (FkDomainSuffixTransformer, DomainSuffix, String, EmptyConfig),
    (FkFreeEmailTransformer, FreeEmail, String, EmptyConfig),
    (FkSafeEmailTransformer, SafeEmail, String, EmptyConfig),
    (FkUsernameTransformer, Username, String, EmptyConfig),
    (FkPasswordTransformer, Password, String, LenConfig),
    (FkIPv4Transformer, IPv4, String, EmptyConfig),
    (FkIPv6Transformer, IPv6, String, EmptyConfig),
    (FkMACAddressTransformer, MACAddress, String, EmptyConfig),
    (FkColorTransformer, Color, String, EmptyConfig),
    (FkUserAgentTransformer, UserAgent, String, EmptyConfig),
    (FkJobSeniorityTransformer, JobSeniority, String, EmptyConfig),
    (FkJobFieldTransformer, JobField, String, EmptyConfig),
    (FkJobPositionTransformer, JobPosition, String, EmptyConfig),
    (FkJobTitleTransformer, JobTitle, String, EmptyConfig),
    (FkWordTransformer, Word, String, EmptyConfig),
    (FkWordsTransformer, Words, Vec<String>, CountConfig),
    (FkSentenceTransformer, Sentence, String, CountConfig),
    (FkSentencesTransformer, Sentences, Vec<String>, CountConfig),
    (FkParagraphTransformer, Paragraph, String, CountConfig),
    (
        FkParagraphsTransformer,
        Paragraphs,
        Vec<String>,
        CountConfig
    ),
    (FkFirstNameTransformer, FirstName, String, EmptyConfig),
    (FkLastNameTransformer, LastName, String, EmptyConfig),
    (FkNameSuffixTransformer, NameSuffix, String, EmptyConfig),
    (FkPersonTitleTransformer, PersonTitle, String, EmptyConfig),
    (FkPersonNameTransformer, PersonName, String, EmptyConfig),
    (
        FkPersonNameWithTitleTransformer,
        PersonNameWithTitle,
        String,
        EmptyConfig
    ),
    (FkDigitTransformer, Digit, String, EmptyConfig),
    (FkPhoneNumberTransformer, PhoneNumber, String, EmptyConfig),
    (FkCellNumberTransformer, CellNumber, String, EmptyConfig),
    (FkFilePathTransformer, FilePath, String, EmptyConfig),
    (FkFileNameTransformer, FileName, String, EmptyConfig),
    (
        FkFileExtensionTransformer,
        FileExtension,
        String,
        EmptyConfig
    ),
    (FkDirPathTransformer, DirPath, String, EmptyConfig),
    (FkCurrencyCodeTransformer, CurrencyCode, String, EmptyConfig),
    (FkCurrencyNameTransformer, CurrencyName, String, EmptyConfig),
    (
        FkCurrencySymbolTransformer,
        CurrencySymbol,
        String,
        EmptyConfig
    )
];

#[cfg(test)]
mod tests {
    use super::*;

    mod deserialization {
        use super::*;

        #[test]
        fn city_name() {
            let cfg = r#"
                locale: EN
                config: ~
                "#;
            let t: FkCityNameTransformer = serde_yaml::from_str(cfg).unwrap();
            assert_eq!(
                t,
                FkCityNameTransformer {
                    locale: LocaleConfig::EN,
                    config: EmptyConfig
                }
            );
        }

        #[test]
        fn boolean() {
            let cfg = r#"
                locale: EN
                config:
                  ratio: 30
                "#;
            let t: FkBooleanTransformer = serde_yaml::from_str(cfg).unwrap();
            assert_eq!(
                t,
                FkBooleanTransformer {
                    locale: LocaleConfig::EN,
                    config: RatioConfig { ratio: 30 }
                }
            );
        }

        #[test]
        fn default_locale() {
            let cfg = "config: ~";
            let t: FkCityNameTransformer = serde_yaml::from_str(cfg).unwrap();
            assert_eq!(
                t,
                FkCityNameTransformer {
                    locale: LocaleConfig::EN,
                    config: EmptyConfig
                }
            );
        }

        #[test]
        fn default_config() {
            let cfg = "locale: RU";
            let t: FkBooleanTransformer = serde_yaml::from_str(cfg).unwrap();
            assert_eq!(
                t,
                FkBooleanTransformer {
                    locale: LocaleConfig::RU,
                    config: RatioConfig::default()
                }
            );
        }

        #[test]
        fn default_all() {
            let cfg = "{}";
            let t: FkBooleanTransformer = serde_yaml::from_str(cfg).unwrap();
            assert_eq!(
                t,
                FkBooleanTransformer {
                    locale: LocaleConfig::default(),
                    config: RatioConfig::default()
                }
            );
        }
    }

    #[test]
    fn boolean() {
        let t = FkBooleanTransformer {
            locale: LocaleConfig::EN,
            config: RatioConfig { ratio: 0 },
        };
        assert_eq!(
            t.transform("table.field", "t", &None),
            Ok(Some(String::from("FALSE")))
        );

        let t = FkBooleanTransformer {
            locale: LocaleConfig::EN,
            config: RatioConfig { ratio: 100 },
        };
        assert_eq!(
            t.transform("table.field", "t", &None),
            Ok(Some(String::from("TRUE")))
        );
    }

    #[test]
    fn city_name() {
        let t = FkCityNameTransformer {
            locale: LocaleConfig::EN,
            config: EmptyConfig,
        };
        let value = t.transform("table.field", "t", &None).unwrap().unwrap();
        assert!(value.len() > 1);
        assert!(('A'..='Z').contains(&value.chars().next().unwrap()));
    }

    #[test]
    fn country_name() {
        let t = FkCityNameTransformer {
            locale: LocaleConfig::EN,
            config: EmptyConfig,
        };
        let value = t.transform("table.field", "t", &None).unwrap().unwrap();
        assert!(value.len() > 1);
        assert!(('A'..='Z').contains(&value.chars().next().unwrap()));
    }

    #[test]
    fn longitude() {
        let t = FkLongitudeTransformer {
            locale: LocaleConfig::EN,
            config: EmptyConfig,
        };
        let value = t.transform("table.field", "t", &None).unwrap().unwrap();
        let value: f32 = value.parse().unwrap();
        assert!(value < 360.0 && value > -360.0);
    }

    #[test]
    fn datetime() {
        let t = FkDateTimeTransformer {
            locale: LocaleConfig::EN,
            config: EmptyConfig,
        };
        let value = t.transform("table.field", "t", &None).unwrap().unwrap();
        // yyyy-mm-dd hh:mm:ss
        assert_eq!(value.len(), 19);
    }

    #[test]
    fn password() {
        let t = FkPasswordTransformer {
            locale: LocaleConfig::EN,
            config: LenConfig {
                len: RangeConfig { min: 10, max: 10 },
            },
        };
        let value = t.transform("table.field", "t", &None).unwrap().unwrap();
        assert_eq!(value.len(), 10);
    }

    #[test]
    fn words() {
        let t = FkWordsTransformer {
            locale: LocaleConfig::EN,
            config: CountConfig {
                count: RangeConfig { min: 5, max: 5 },
            },
        };
        let value = t.transform("table.field", "t", &None).unwrap().unwrap();
        assert_eq!(value.split(" ").count(), 5);
    }
}
