# [Data]nymizer

Simple data anonymizer with flexible rules.

## Supported databases

- [x] Postgresql
- [ ] MySQL or MariaDB (TODO)

## How it works

Database -> Dumper (+Faker) -> Dump.sql

## Available rules

| Rule          | Description                                                                  |
|---------------|------------------------------------------------------------------------------|
| email         | Emails with different options                                                |
| ip            | IP addresses. Supports IPv4 and IPv6                                         |
| password      | Password with different <br>length options (support `max` and `min` options) |
| words         | Lorem words with different length                                            |
| city          | City names generator                                                         |
| datetime      | Make DateTime strings with options (`from` and `to`)                         |
| first_name    | First name generator                                                         |
| last_name     | Last name generator                                                          |
| digit         | Random digit (in range `0..9`)                                               |
| random_number | Random number with `min` and `max` options                                   |
| phone         | Generate random phone with different `format`                                |
| template      | Template engine for renerate random text with included rules                  |
