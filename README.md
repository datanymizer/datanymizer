# [Data]nymizer

Simple data anonymizer with flexible rules.

## How it works

Database -> Dumper (+Faker) -> Dump.sql

You can import or process you dump with supported database witout 3rd-party importers.

Datanymizer generate database-native dump.

## Available rules

| Rule            | Description                                                                  |
|-----------------|------------------------------------------------------------------------------|
| `email`         | Emails with different options                                                |
| `ip`            | IP addresses. Supports IPv4 and IPv6                                         |
| `words`         | Lorem words with different length                                            |
| `first_name`    | First name generator                                                         |
| `last_name`     | Last name generator                                                          |
| `city`          | City names generator                                                         |
| `phone`         | Generate random phone with different `format`                                |
| `pipeline`      | Use pipeline to generate more difficult values                               |
| `capitalize`    | Like filter, it capitalize input value                                       |
| `template`      | Template engine for generate random text with included rules                 |
| `digit`         | Random digit (in range `0..9`)                                               |
| `random_number` | Random number with `min` and `max` options                                   |
| `password`      | Password with different <br>length options (support `max` and `min` options) |
| `datetime`      | Make DateTime strings with options (`from` and `to`)                         |
| and more...     |                                                                              |

## Supported databases

- [x] Postgresql
- [ ] MySQL or MariaDB (TODO)
