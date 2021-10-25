# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### 🚀 Added
- Add support for `postgresql` scheme [#115](https://github.com/datanymizer/datanymizer/pull/115)
  ([@mgrachev](https://github.com/mgrachev))

### ⚙️ Changed
- Change edition to 2021 [#113](https://github.com/datanymizer/datanymizer/pull/113)
  ([@mgrachev](https://github.com/mgrachev))

### 🛠 Fixed
- Replace `unwrap_or_else` with `unwrap_or_default` [#112](https://github.com/datanymizer/datanymizer/pull/112)
  ([@mgrachev](https://github.com/mgrachev))

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
- Implement the most of fakers from the `fake` crate. Add the option for locale configuration
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
