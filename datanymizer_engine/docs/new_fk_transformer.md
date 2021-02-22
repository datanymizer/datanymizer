# How to add a new faker-based transformer

To create transformers, we actively use the [fake](https://github.com/cksac/fake-rs) crate.
It is a Rust library for generating fake data.

Fakers must implement the `fake::Fake` trait. It supports using the locale as an argument for internalization. 
You can implement the `FkTransformer` trait to easily interact with the `fake::Fake` trait in your transformer.

You can implement `FkTransformer` manually (if you want to place your transformer in a separate module, perhaps in 
separate crate) or you can add some data to the macro in the
[transformers/fk/mod.rs](/datanymizer_engine/src/transformers/fk/mod.rs) file. We use this macro to reduce the size of
boilerplate code.

Let's suppose that we want to implement a transformer that uses some hypothetical `Passport` faker to generate
passport data.

Fakers are tuple structs with the locale as the first field. 
They can return values of different types (in our example, we assume `String`).

```rust
use passport::Passport;

fn main() {
    let value: String = Passport(EN).fake();
    println!("{}", value);
}
```

The type of return value must implement the `AsSqlValue` trait. This trait converts the values to `String` for output to
SQL-dump. It is already implemented for `bool`, `String`, `Vec<String>`, `GenericInt` (alias for `isize`),
`GenericFloat` (alias for `f64`), `GenericDate` (alias for `chrono::naive::NaiveDate`) and `GenericDateTime` (alias for
`chrono::naive::NaiveDateTime`).

You can easily implement it for other types (refer to the
[transformers/fk/sql_value.rs](/datanymizer_engine/src/transformers/fk/sql_value.rs) file).

## Defining the transformer in a separate module

```rust
use fake::Fake;
use serde::{Deserialize, Serialize};

// This is not a real faker
use passport::Passport;

use datanymizer_engine::{
    FkTransformer, LocaleConfig, Localized, LocalizedFaker, TransformContext, TransformResult, Transformer,
    TransformerDefaults,
};

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
#[serde(default)]
pub struct PassportTransformer {
    pub locale: Option<LocaleConfig>,
}

impl Localized for PassportTransformer {
    fn locale(&self) -> Option<LocaleConfig> {
        self.locale
    }

    fn set_locale(&mut self, l: Option<LocaleConfig>) {
        self.locale = l;
    }
}

impl LocalizedFaker<String> for PassportTransformer {
    fn fake<L: Copy + fake::locales::Data>(&self, l: L) -> String {
        Passport(l).fake()
    }
}

impl FkTransformer<String> for PassportTransformer {}

impl Transformer for PassportTransformer {
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
```

## Defining the transformer with the macros from `transformers::fk` module

Fakers have a different structure. We currently support the following structures:

* `Empty` - `SomeFaker(EN)`, a faker struct has a single locale field (like our `Passport`).
  
* `Ratio` - `SomeFaker(EN, ratio)`, a faker struct has an additional `u8` field, the `fake::faker::raw::Boolean` faker
  has this structure (`ratio` is a probability of the `true` value).
  
* `Count` - `SomeFaker(EN, count)`, a faker struct has an additional range field (from min to max). For example, the  
  `fake::faker::lorem::raw::Words` faker has this structure (`count` determines the number of words).

It is easier to add a faker with one of these structures.

### With the already implemented structure

1. In the [transformers/fk/mod.rs](/datanymizer_engine/src/transformers/fk/mod.rs) file.
   
Add dependency for the faker:

```rust
use passport::Passport;
```
    
Update the transformer list in the `define_fk_transformers!` macro call:

```rust
define_fk_transformers![
    "Gets a city name.",
    ("city", CityTransformer, CityName, String, Empty),
    "Gets a city prefix (e.g., `North`- or `East`-).",
    ("city_prefix", CityPrefixTransformer, CityPrefix, String, Empty),
    "Gets a city suffix (e.g., -`town`, -`berg` or -`ville`).",
    ("city_suffix", CitySuffixTransformer, CitySuffix, String, Empty),
    // ......
    // Our transformer
     "Gets a passport data.",
    ("passport", PassportTransformer, Passport, String, Empty)
];
```

2. In the [transformers/mod.rs](/datanymizer_engine/src/transformers/mod.rs) file.

Add our transformer to the `define_transformers_enum!` macro call:

```rust
define_transformers_enum![
    ("none", None, NoneTransformer),
    ("email", Email, EmailTransformer),
    ("ip", IP, IpTransformer),
    ("phone", Phone, PhoneTransformer),
    // ......
    // Our transformer
    ("passport", Passport, PassportTransformer)
];
```

### With a new structure

Let's imagine that our faker has the additional numeric field that determines the year when a passport was issued:

```rust
use passport::Passport;

fn main() {
    let value: String = Passport(EN, 2010).fake();
    println!("{}", value);
}
```

The code you add in the step 1 (update the transformer list...) will look like this:

```rust
define_fk_transformers![
    "Gets a city name.",
    ("city", CityTransformer, CityName, String, Empty),
    "Gets a city prefix (e.g., `North`- or `East`-).",
    ("city_prefix", CityPrefixTransformer, CityPrefix, String, Empty),
    "Gets a city suffix (e.g., -`town`, -`berg` or -`ville`).",
    ("city_suffix", CitySuffixTransformer, CitySuffix, String, Empty),
    // ......
    // Our transformer
     "Gets a passport data.",
    ("passport", PassportTransformer, Passport, String, Year)
];
```

Then you have to make the following additional changes in the
[transformers/fk/mod.rs](/datanymizer_engine/src/transformers/fk/mod.rs) file:

Add this code to the `fk_config_example!` macro.

```rust
macro_rules! fk_config_example {
   // .........
   
   ( Year ) => {
       concat!(
       "      # Year\n",
       "      year: 2000\n",

       )
   };
}
```

Add this code to the `define_fk_struct!` macro.

```rust
macro_rules! define_fk_struct {
    // .........
    
    ( $tr:ident, Year, $doc:expr ) => {
        #[doc = $doc]
        #[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
        #[serde(default)]
        pub struct $tr {
            pub locale: LocaleConfig,
            pub year: isize,            
        }

        impl Default for $tr {
            fn default() -> Self {
                Self {
                    locale: LocaleConfig::default(),
                    year: 2000,
                }
            }
        }
    };
}
```

Add this code to the `impl_localized_faker!` macro.

```rust
macro_rules! impl_localized_faker {
    // .........
    
    ( $fk:ident, $sql:ty, Year ) => {
        fn fake<L: Copy + fake::locales::Data>(&self, l: L) -> $sql {
            $fk(l, self.year).fake()
        }
    };
}
```
