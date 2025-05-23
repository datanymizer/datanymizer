# [Data]nymizer

<img align="right" 
     alt="datanymizer"
     src="https://raw.githubusercontent.com/datanymizer/datanymizer/master/logo.png">

[![Build Status](https://github.com/datanymizer/datanymizer/actions/workflows/ci.yml/badge.svg)](https://github.com/datanymizer/datanymizer/actions/workflows/ci.yml)
![License](https://img.shields.io/github/license/datanymizer/datanymizer)
![Release Version](https://img.shields.io/github/v/release/datanymizer/datanymizer)
[![CodeCov](https://codecov.io/gh/datanymizer/datanymizer/branch/main/graph/badge.svg)](https://app.codecov.io/gh/datanymizer/datanymizer)
[![Audit](https://github.com/datanymizer/datanymizer/actions/workflows/audit.yml/badge.svg)](https://github.com/datanymizer/datanymizer/actions/workflows/audit.yml)

Powerful database anonymizer with flexible rules. Written in Rust.

Datanymizer is created & [supported by Evrone](https://evrone.com/?utm_campaign=datanymizer). See what else we [develop with Rust](https://evrone.com/rust?utm_source=github&utm_campaign=datanymizer).

More information you can find in articles in [English](https://evrone.com/datanymizer?utm_source=github&utm_campaign=datanymizer) and [Russian](https://evrone.ru/datanymizer?utm_source=github&utm_campaign=datanymizer).

## How it works

Database -> Dumper (+Faker) -> Dump.sql

You can import or process your dump with supported database without 3rd-party importers.

Datanymizer generates database-native dump.

## Installation

There are several ways to install `pg_datanymizer`, choose a more convenient option for you.

### Pre-compiled binary

```bash
# Linux / macOS / Windows (MINGW and etc). Installs it into ./bin/ by default
$ curl -sSfL https://raw.githubusercontent.com/datanymizer/datanymizer/main/cli/pg_datanymizer/install.sh | sh -s

# Or more shorter way
$ curl -sSfL https://git.io/pg_datanymizer | sh -s

# Specify installation directory and version
$ curl -sSfL https://git.io/pg_datanymizer | sudo sh -s -- -b /usr/local/bin v0.2.0

# Alpine Linux (wget)
$ wget -q -O - https://git.io/pg_datanymizer | sh -s
```

#### Homebrew / Linuxbrew

```bash
# Installs the latest stable release
$ brew install datanymizer/tap/pg_datanymizer

# Builds the latest version from the repository
$ brew install --HEAD datanymizer/tap/pg_datanymizer
```

#### Docker

```bash
$ docker run --rm -v `pwd`:/app -w /app datanymizer/pg_datanymizer
```

## Getting started with CLI dumper

First, inspect your database schema, choose fields with sensitive data, and create a config file based on it.

```yaml
# config.yml
tables:
  - name: markets
    rules:
      name_translations:
        template:
          format: '{"en": "{{_1}}", "ru": "{{_2}}"}'
          rules:
            - words:
                min: 1
                max: 2
            - words:
                min: 1
                max: 2
  - name: franchisees
    rules:
      operator_mail:
        template:
          format: user-{{_1}}-{{_2}}
          rules:
            - random_num: {}
            - email:
                kind: Safe
      operator_name:
        first_name: {}
      operator_phone:
        phone:
          format: +###########
      name_translations:
        template:
          format: '{"en": "{{_1}}", "ru": "{{_2}}"}'
          rules:
            - words:
                min: 2
                max: 3
            - words:
                min: 2
                max: 3
  - name: users
    rules:
      first_name:
        first_name: {}
      last_name:
        last_name: {}
  - name: customers
    rules:
      email:
        template:
          format: user-{{_1}}-{{_2}}
          rules:
            - random_num: {}
            - email:
                kind: Safe
                uniq:  
                  required: true
                  try_count: 5
      phone:
        phone:
          format: +7##########
          uniq: true
      city:
        city: {}
      age:
        random_num:
          min: 10
          max: 99
      first_name:
        first_name: {}
      last_name:
        last_name: {}
      birth_date:
        datetime:
          from: 1990-01-01T00:00:00+00:00
          to: 2010-12-31T00:00:00+00:00
```

And then start to make dump from your database instance:

```bash
pg_datanymizer -f /tmp/dump.sql -c ./config.yml postgres://postgres:postgres@localhost/test_database
```

It creates new dump file `/tmp/dump.sql` with native SQL dump for Postgresql database.
You can import fake data from this dump into new Postgresql database with command:

```bash
psql -U postgres -d new_database < /tmp/dump.sql
```

Dumper can stream dump to `STDOUT` like `pg_dump` and you can use it in other pipelines:

```bash
pg_datanymizer -c ./config.yml postgres://postgres:postgres@localhost/test_database > /tmp/dump.sql
```


## Additional options

### Tables filter

You can specify which tables you choose or ignore for making dump.

For dumping only `public.markets` and `public.users` data.

```yaml
# config.yml
#...
filter:
  only:
    - public.markets
    - public.users
```

For ignoring those tables and dump data from others.

```yaml
# config.yml
#...
filter:
  except:
    - public.markets
    - public.users
```

You can also specify data and schema filters separately.

This is equivalent to the previous example.

```yaml
# config.yml
#...
filter:
  data:
    except:
      - public.markets
      - public.users
```

For skipping schema and data from other tables.

```yaml
# config.yml
#...
filter:
  schema:
    only:
      - public.markets
      - public.users
```

For skipping schema for `markets` table and dumping data only from `users` table.

```yaml
# config.yml
#...
filter:
  data:
    only:
      - public.users
  schema:
    except:
      - public.markets
```

You can use wildcards in the `filter` section:

* `?` matches exactly one occurrence of any character;
* `*` matches arbitrary many (including zero) occurrences of any character.

### Dump conditions and limit

You can specify conditions (SQL `WHERE` statement) and limit for dumped data per table:

```yaml
# config.yml
tables:
  - name: people
    query:
      # don't dump some rows
      dump_condition: "last_name <> 'Sensitive'"
      # select maximum 100 rows
      limit: 100 
```

### Transform conditions and limit

As the additional option, you can specify SQL conditions that define which rows will be transformed (anonymized):

```yaml
# config.yml
tables:
  - name: people
    query:
      # don't dump some rows
      dump_condition: "last_name <> 'Sensitive'"
      # preserve original values for some rows
      transform_condition: "NOT (first_name = 'John' AND last_name = 'Doe')"      
      # select maximum 100 rows
      limit: 100
```

You can use the `dump_condition`, `transform_condition` and `limit` options in any combination (only
`transform_condition`; `transform_condition` and `limit`; etc).

### Global variables

You can specify global variables available from any `template` rule.

```yaml
# config.yml
tables:
  users:
    bio:
      template:
        format: "User bio is {{var_a}}"
    age:
      template:
        format: {{_0 | float * global_multiplicator}}
#...
globals:
  var_a: Global variable 1
  global_multiplicator: 6
```

## Available rules

| Rule                           | Description                                                                  |
|--------------------------------|------------------------------------------------------------------------------|
| `email`                        | Emails with different options                                                |
| `ip`                           | IP addresses. Supports IPv4 and IPv6                                         |
| `words`                        | Lorem words with different length                                            |
| `first_name`                   | First name generator                                                         |
| `last_name`                    | Last name generator                                                          |
| `city`                         | City names generator                                                         |
| `phone`                        | Generate random phone with different `format`                                |
| `pipeline`                     | Use pipeline to generate more complicated values                             |
| `capitalize`                   | Like filter, it capitalizes input value                                      |
| `template`                     | Template engine for generate random text with included rules                 |
| `digit`                        | Random digit (in range `0..9`)                                               |
| `random_num`                   | Random number with `min` and `max` options                                   |
| `password`                     | Password with different <br>length options (support `max` and `min` options) |
| `datetime`                     | Make DateTime strings with options (`from` and `to`)                         |
| more than 70 rules in total... |                                                                              |

For the complete list of rules please refer [this document](docs/transformers.md).

### Uniqueness

You can specify that result values must be unique (they are not unique by default).
You can use short or full syntax.

Short:
```yaml
uniq: true
```

Full:
```yaml
uniq:
  required: true
  try_count: 5
```

Uniqueness is ensured by re-generating values when they are same.
You can customize the number of attempts with `try_count` (this is an optional field, the default number of tries
depends on the rule).

Currently, uniqueness is supported by: `email`, `ip`, `phone`, `random_num`.

### Locales

You can specify the locale for individual rules:

```yaml
first_name:
  locale: RU
```

The default locale is `EN` but you can specify a different default locale:

```yaml
tables:
  # ........  
default:
  locale: RU
```

We also support `ZH_TW` (traditional chinese) and `RU` (translation in progress).

## Referencing row values from templates

You can reference values of other row fields in templates.
Use `prev` for original values and `final` - for anonymized:

```yaml
tables:
  - name: some_table
    # You must specify the order of rule execution when using `final`
    rule_order:
      - greeting
      - options
    rules:
      first_name:
        first_name: {}
      greeting:
        template:
          # Keeping the first name, but anonymizing the last name   
          format: "Hello, {{ prev.first_name }} {{ final.last_name }}!"
      options:
        template:
          # Using the anonymized value again   
          format: "{greeting: \"{{ final.greeting }}\"}"
```

You must specify the order of rule execution when using `final` with `rule_order`.
All rules not listed will be placed at the beginning (i.e. you must list only rules with `final`).

## Sharing information between rows

We implemented a built-in key-value store that allows information to be exchanged between anonymized rows.

It is available via the special functions in templates.

Take a look at an example:

```yaml
tables:
  - name: users  
    rules:
      name:
        template:    
          # Save a name to the store as a side effect, the key is `user_names.<USER_ID>` 
          format: "{{ _1 }}{{ store_write(key='user_names.' ~ prev.id, value=_1) }}"
          rules:
            - person_name: {}
  - name: user_operations
    rules:
      user_name:          
        template:
          # Using the saved value again  
          format: "{{ store_read(key='user_names.' ~ prev.user_id) }}"
```

## Supported databases

- [x] Postgresql
- [ ] MySQL or MariaDB (TODO)

## Documentation

* [pg_datanymizer](docs/pg_datanymizer.md) CLI application manual.
* [config.yml](docs/config.md) file specification.
* [Full list](docs/transformers.md) of transformation rules.
* [Integration testing](docs/integration_tests.md) manual.

## Sponsors

<p>
  <a href="https://evrone.com/?utm_source=github&utm_campaign=datanymizer">
    <img src="https://camo.githubusercontent.com/433f193098927e4e7229c229c8920f77898282063d4fc3cbafb04ea3d24d73df/68747470733a2f2f6576726f6e652e636f6d2f6c6f676f2f6576726f6e652d73706f6e736f7265642d6c6f676f2e706e67"
      alt="Sponsored by Evrone" width="210">
  </a>
</p>

## License

[MIT](https://github.com/datanymizer/datanymizer/blob/main/LICENSE)

## Development

### Cross compilation

Mac to Linux

```
rustup target add x86_64-unknown-linux-gnu
brew tap messense/macos-cross-toolchains
brew install x86_64-unknown-linux-gnu
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc cargo build --target x86_64-unknown-linux-gnu --release --features openssl/vendored
```
