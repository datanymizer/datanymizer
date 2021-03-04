# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### 🚀 Added
- Referencing values from original and transformed row. Improve performance for the TemplateTransformer
  [#41](https://github.com/datanymizer/datanymizer/pull/41) ([@evgeniy-r](https://github.com/evgeniy-r))
- Add a config-wide default locale (and the method to set defaults for transformers)
  [#36](https://github.com/datanymizer/datanymizer/pull/36) ([@evgeniy-r](https://github.com/evgeniy-r))
- Implement the most of fakers from the `fake` crate. Add the option for locale configuration
  [#23](https://github.com/datanymizer/datanymizer/pull/23) ([@evgeniy-r](https://github.com/evgeniy-r))

### ⚙️ Changed

### 🛠 Fixed
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
