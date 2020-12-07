# [Data]nymizer

Simple data anonymizer with flexible rules.

## How it works

Database -> Dumper (+Faker) -> Dump.sql

You can import or process you dump with supported database without 3rd-party importers.

Datanymizer generates database-native dump.

## Getting started with CLI dumper

Build supported database dumper (`pg_dump_faker` for example).

``` shell
cargo build --release
```

Inspect your database schema, choose fields with sensitive data and create config, based on it.

``` yaml
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
        first_name: ~
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
        first_name: ~
      last_name:
        last_name: ~
  - name: customers
    rules:
      email:
        template:
          format: user-{{_1}}-{{_2}}
          rules:
            - random_num: {}
            - email:
                kind: Safe
      phone:
        phone:
          format: +7##########
          uniq: true
      city:
        city: ~
      age:
        random_num:
          min: 10
          max: 99
      first_name:
        first_name: ~
      last_name:
        last_name: ~
      birth_date:
        datetime:
          from: 1990-01-01T00:00:00+00:00
          to: 2010-12-31T00:00:00+00:00
```

And then start to make dump from your database instance:

``` shell
./pg_dump_faker -f /tmp/dump.sql -c ./config.yml postgres://postgres:postgres@localhost/test_database
```

It creates new dump file `/tmp/dump.sql` with native SQL dump for Postgresql database.
You can import fake data from this dump into new Postgresql database with command:

``` shell
psql -Upostgres -d new_database < /tmp/dump.sql
```

## Additional options

### Tables filter

You can specify which tables you choose or ignore for making dump.

``` yaml
# config.yml
#...
filter:
  only:
    - public.markets
    - public.users
```

For dumping only `public.markets` and `public.users` data.

``` yaml
# config.yml
#...
filter:
  except:
    - public.markets
    - public.users
```

For ignoring those tables and dump data from others.

### Global variables

You can specify global variables available from any `template` rule.

``` yaml
# config.yml
tables:
  users:
    bio:
      template:
        format: "User bio is {{var_a}}"
    age:
      template:
        format: {{_0 * global_multiplicator}}
#...
globals:
  var_a: Global variable 1
  global_multiplicator: 6
```

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
| `capitalize`    | Like filter, it capitalizes input value                                       |
| `template`      | Template engine for generate random text with included rules                 |
| `digit`         | Random digit (in range `0..9`)                                               |
| `random_number` | Random number with `min` and `max` options                                   |
| `password`      | Password with different <br>length options (support `max` and `min` options) |
| `datetime`      | Make DateTime strings with options (`from` and `to`)                         |
| and more...     |                                                                              |

## Supported databases

- [x] Postgresql
- [ ] MySQL or MariaDB (TODO)

## Sponsors

<p>
  <a href="https://evrone.com/?utm_source=github&utm_campaign=datanymizer">
    <img src="https://camo.githubusercontent.com/433f193098927e4e7229c229c8920f77898282063d4fc3cbafb04ea3d24d73df/68747470733a2f2f6576726f6e652e636f6d2f6c6f676f2f6576726f6e652d73706f6e736f7265642d6c6f676f2e706e67"
      alt="Sponsored by Evrone" width="210">
  </a>
</p>

## License

[MIT](https://choosealicense.com/licenses/mit)
