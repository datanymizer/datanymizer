# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### 🚀 Added
- Ignore computed columns (previously they would cause an error). [#202](https://github.com/datanymizer/datanymizer/pull/202)
  ([@gregwebs](https://github.com/gregwebs))
- Add the JSON transformer [#134](https://github.com/datanymizer/datanymizer/pull/134)
  ([@evgeniy-r](https://github.com/evgeniy-r))

### ⚙️ Changed

### 🛠 Fixed

## [v0.6.0] - 2022-08-09
### 🚀 Added
- Add the ability to select verbose logging features [#184](https://github.com/datanymizer/datanymizer/pull/184)
  ([@akirill0v](https://github.com/akirill0v))
- Add the UUID transformer [#180](https://github.com/datanymizer/datanymizer/pull/180)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Add the plain text transformer [#177](https://github.com/datanymizer/datanymizer/pull/177)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Add bcrypt filter to generate hashes [#164](https://github.com/datanymizer/datanymizer/pull/164)
  ([@akirill0v](https://github.com/akirill0v))
- Add wildcards support in the filter section [#151](https://github.com/datanymizer/datanymizer/pull/151)
  ([@evgeniy-r](https://github.com/evgeniy-r))

### ⚙️ Changed
- Use forked Tera (because of a security issue with `chrono`)
  [#171](https://github.com/datanymizer/datanymizer/pull/171) ([@evgeniy-r](https://github.com/evgeniy-r))
- Speed up loading of metadata [#170](https://github.com/datanymizer/datanymizer/pull/170)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Refactoring (more generic `Dumper` trait) [#159](https://github.com/datanymizer/datanymizer/pull/159)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Remove the dependency from `chrono` ([security issue](https://github.com/chronotope/chrono/pull/578)).
  Remove the `raw_date` and `raw_datetime` transformers (duplication with the `datetime` transformer)
  [#152](https://github.com/datanymizer/datanymizer/pull/152) ([@evgeniy-r](https://github.com/evgeniy-r)).

### 🛠 Fixed
- Update the `config` and `ignore` crates (security issues) [#176](https://github.com/datanymizer/datanymizer/pull/176)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Update the `regex` crate (security issue) [#172](https://github.com/datanymizer/datanymizer/pull/172)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Use pg_namespace to deconflict duplicate field names [#149](https://github.com/datanymizer/datanymizer/pull/149)
  ([@mbeynon](https://github.com/mbeynon))
- Fix Postgres COPY syntax when dumping a table with zero defined fields
  [#147](https://github.com/datanymizer/datanymizer/pull/147) ([@mbeynon](https://github.com/mbeynon))
- Fix the bug with a datetime format [#150](https://github.com/datanymizer/datanymizer/pull/150)
  ([@evgeniy-r](https://github.com/evgeniy-r))

## [v0.5.1] - 2021-12-05
### 🚀 Added
- Add integration tests [#130](https://github.com/datanymizer/datanymizer/pull/130)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Add the `default` arg for `store_read` template function [#133](https://github.com/datanymizer/datanymizer/pull/133)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Add the configurable table order [#127](https://github.com/datanymizer/datanymizer/pull/127)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Extend template transformer with shared templates and macros
  [#122](https://github.com/datanymizer/datanymizer/pull/122) ([@akirill0v](https://github.com/akirill0v))
- Testing the demo [#114](https://github.com/datanymizer/datanymizer/pull/114)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Add support for `postgresql` scheme [#115](https://github.com/datanymizer/datanymizer/pull/115)
  ([@mgrachev](https://github.com/mgrachev))

### ⚙️ Changed
- Refactoring: generic indicator and writer [#132](https://github.com/datanymizer/datanymizer/pull/132)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Refactoring: remove unrelated code from Engine [#131](https://github.com/datanymizer/datanymizer/pull/131)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Remove the PostgreSQL dependency from the CLI application [#123](https://github.com/datanymizer/datanymizer/pull/123)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Remove arch-specific argument in Demo [#121](https://github.com/datanymizer/datanymizer/pull/121)
  ([@akirill0v](https://github.com/akirill0v))
- Change edition to 2021 [#113](https://github.com/datanymizer/datanymizer/pull/113)
  ([@mgrachev](https://github.com/mgrachev))

### 🛠 Fixed
- Replace `unwrap_or_else` with `unwrap_or_default` [#112](https://github.com/datanymizer/datanymizer/pull/112)
  ([@mgrachev](https://github.com/mgrachev))

## [v0.5.0] - 2021-12-05

The release was yanked.

## [v0.4.0] - 2021-10-19
### 🚀 Added
- Implement returning NULL values for PostgreSQL from transformers
  [#98](https://github.com/datanymizer/datanymizer/pull/98) ([@evgeniy-r](https://github.com/evgeniy-r))
- Configurable dump transaction (whether to use, an isolation level)
  [#96](https://github.com/datanymizer/datanymizer/pull/96) ([@evgeniy-r](https://github.com/evgeniy-r))

### 🛠 Fixed
- Fix problems with different table names [#102](https://github.com/datanymizer/datanymizer/pull/102)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Fix demo docker environment [#109](https://github.com/datanymizer/datanymizer/pull/109)
  ([@mgrachev](https://github.com/mgrachev))

## [v0.3.1] - 2021-09-20
### 🚀 Added
- `ExtData` trait, dictionaries for the Russian locale (person and company names), the new transformers: `middle_name`
  and `company_name_alt` [#83](https://github.com/datanymizer/datanymizer/pull/83)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Add sequences to the dump [#84](https://github.com/datanymizer/datanymizer/pull/84)
  ([@evgeniy-r](https://github.com/evgeniy-r)) 
- Add `prefix` and `suffix` options for `EmailTransformer` [#80](https://github.com/datanymizer/datanymizer/pull/80)
  ([@evgeniy-r](https://github.com/evgeniy-r)) 
- Additional arguments for pg_dump [#78](https://github.com/datanymizer/datanymizer/pull/78)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- `base64_token` and `base64url_token` transformers. [#77](https://github.com/datanymizer/datanymizer/pull/77)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Key-value store that allows to share information between template transformers
  [#75](https://github.com/datanymizer/datanymizer/pull/75) ([@evgeniy-r](https://github.com/evgeniy-r))
- Options to ignore invalid certificates and hostnames when using SSL
  [#64](https://github.com/datanymizer/datanymizer/pull/64) ([@evgeniy-r](https://github.com/evgeniy-r))
- Improve docs [#67](https://github.com/datanymizer/datanymizer/pull/67) ([@evgeniy-r](https://github.com/evgeniy-r))
- Add the basic SSL support for Postgres [#61](https://github.com/datanymizer/datanymizer/pull/61)
  ([@evgeniy-r](https://github.com/evgeniy-r))

### ⚙️ Changed
- Move `rnd_chars()` from `token` to `utils` [#79](https://github.com/datanymizer/datanymizer/pull/79)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Change transformer initialization (`set_defaults` -> `init`) [#76](https://github.com/datanymizer/datanymizer/pull/76)
  ([@evgeniy-r](https://github.com/evgeniy-r))

### 🛠 Fixed
- Fix a release Cargo profile [#89](https://github.com/datanymizer/datanymizer/pull/89)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Fix the audit pipeline (using fresh bases) [#88](https://github.com/datanymizer/datanymizer/pull/88)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Update the `tokio` and `futures` crates due to security issues
  [#87](https://github.com/datanymizer/datanymizer/pull/87) ([@evgeniy-r](https://github.com/evgeniy-r))
- Update the `fake` crate to 2.4.1 [#85](https://github.com/datanymizer/datanymizer/pull/85)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Exit on `pg_dump` errors [#73](https://github.com/datanymizer/datanymizer/pull/73)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Fix EmailTransformer [#72](https://github.com/datanymizer/datanymizer/pull/72)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Fix bug with dropped columns [#69](https://github.com/datanymizer/datanymizer/pull/69)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Update crates with security issues [#65](https://github.com/datanymizer/datanymizer/pull/65)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Add proper escaping after the transformation for Postgres [#60](https://github.com/datanymizer/datanymizer/pull/60)
  ([@evgeniy-r](https://github.com/evgeniy-r))

## [v0.3.0] - 2021-09-20

The release was yanked.

## [v0.2.0] - 2021-04-28
### 🚀 Added
- `hex_token` rule (`HexTokenTransformer`) [#50](https://github.com/datanymizer/datanymizer/pull/50)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Configuration options for transformation SQL conditions [#45](https://github.com/datanymizer/datanymizer/pull/45)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Configuration options for dump query conditions and limit [#47](https://github.com/datanymizer/datanymizer/pull/47)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Referencing values from original and transformed row. Improve performance for the TemplateTransformer
  [#41](https://github.com/datanymizer/datanymizer/pull/41) ([@evgeniy-r](https://github.com/evgeniy-r))
- Add a config-wide default locale (and the method to set defaults for transformers)
  [#36](https://github.com/datanymizer/datanymizer/pull/36) ([@evgeniy-r](https://github.com/evgeniy-r))
- Implement most of the fakers from the `fake` crate. Add the option for locale configuration
  [#23](https://github.com/datanymizer/datanymizer/pull/23) ([@evgeniy-r](https://github.com/evgeniy-r))

### ⚙️ Changed
- Now the `max` option for PasswordTransformer is inclusive [#49](https://github.com/datanymizer/datanymizer/pull/49)
  ([@evgeniy-r](https://github.com/evgeniy-r))

### 🛠 Fixed
- Fix CapitalizeTransformer
  [#44](https://github.com/datanymizer/datanymizer/pull/44) ([@evgeniy-r](https://github.com/evgeniy-r))
- Fix config examples in docs
  [#39](https://github.com/datanymizer/datanymizer/pull/39) ([@evgeniy-r](https://github.com/evgeniy-r))

## [v0.1.0] - 2021-01-27
### 🚀 Added
- Separate filtering of data and schema [#22](https://github.com/datanymizer/datanymizer/pull/22)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Add installation script [#21](https://github.com/datanymizer/datanymizer/pull/21) ([@mgrachev](https://github.com/mgrachev))
- Add the uniqueness option for email, ip and random_num [#16](https://github.com/datanymizer/datanymizer/pull/16)
  ([@evgeniy-r](https://github.com/evgeniy-r))
- Add demo configuration and docker-compose example [#20](https://github.com/datanymizer/datanymizer/pull/20) ([@akirill0v](https://github.com/akirill0v))

### ⚙️ Changed
- Automate release process [#17](https://github.com/datanymizer/datanymizer/pull/17) ([@mgrachev](https://github.com/mgrachev))
- Set up CI [#12](https://github.com/datanymizer/datanymizer/pull/12) ([@mgrachev](https://github.com/mgrachev))
- Rename `pg_dump_faker` to `pg_datanymizer` [#11](https://github.com/datanymizer/datanymizer/pull/11) ([@mgrachev](https://github.com/mgrachev))
