+++
title = "First start"
description = "Getting started with CLI dumper"
date = 2021-05-01T08:20:00+00:00
updated = 2021-05-01T08:20:00+00:00
draft = false
weight = 30
sort_by = "weight"
template = "docs/page.html"

[extra]
lead = ""
toc = true
top = false
+++

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

It creates new dump file <code>/tmp/dump.sql</code> with native SQL dump for Postgresql database. You can import fake data from this dump into new Postgresql database with command:

```bash
psql -U postgres -d new_database < /tmp/dump.sql

```

Dumper can stream dump to <code>STDOUT</code> like <code>pg_dump</code> and you can use it in other pipelines:

```bash
pg_datanymizer -c ./config.yml postgres://postgres:postgres@localhost/test_database > /tmp/dump.sql

```
