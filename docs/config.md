# Configuration file specification

Datanymizer uses a configuration file (`config.yml`) to determine what data to dump and how to anonymize it.

A config example (for the Postgres demo database [DVD Rental](https://sp.postgresqltutorial.com/wp-content/uploads/2019/05/dvdrental.zip)):

```yaml
tables:
  - name: actor
    rules:
      first_name:
        # random name
        first_name: {}
      last_name:
        # random surname
        last_name: {}
      last_update:
        # random date
        datetime:
          from: 1990-01-01T00:00:00+00:00
          to: 2010-12-31T00:00:00+00:00
    query:
      # keeping data of the actor Jane Jackman unanonymized
      transform_condition: "NOT (first_name = 'Jane' AND last_name = 'Jackman')"
      # not dumping the actor with actor_id = 132 (Adam Hopper)
      dump_condition: "actor_id <> 132"

  - name: address
    rules:
      address:
        # using template
        template:
          # using transformed (anonymized) value of district
          format: "{{ final.district }}, {{ _1 }}, {{ _2 }}"
          rules:
            # random street name
            - street_name: {}
            # random building number
            - building_number: {}
      address2:
        # using the template engine (Tera, it is very similar to Jinja) features: condition and built-in function:
        # we add an address comment to roughly half of the rows
        # the template engine is very agile
        template:
          format: "{% if get_random(start=1, end=2) == 1 %}Comment: {{ _1 }}{% endif %}"
          rules:
            # lorem ipsum words (the number of words is 1-2)
            - words:
                min: 1
                max: 2
      district:
        template:
          format: "{{ _1 }}, {{ _2 }}"
          rules:
            # nested template
            - template:
                format: "{{ _2 }} ({{ _1 }})"
                rules:
                  # random country code
                  - country_code: {}
                  # random state abbreviation
                  - state_abbr: {}
            - template:
                format: "dst"
      phone:
        # random phone with some format
        phone:
          format: "7900#######"
          # phones will be unique
          uniq: true
      postal_code:
        # random postal code
        post_code: {}
    # you must specify the order of rule execution when using `final`
    rule_order:
      - address

  - name: city
    rules:
      city:
        city: {}

  - name: customer
    rules:
      active:
        # using anonymized `activebool` value
        template:
          format: "{% if final.activebool == 'TRUE' %}1{% else %}0{% endif %}"
      activebool:
        # the probability of `true` is 80%
        boolean:
          ratio: 80
      create_date:
        datetime:
          from: 2000-01-01T00:00:00+00:00
          to: 2020-12-31T00:00:00+00:00
      email:
        # using the original first name value in the anonymized email
        # also using the anonymized value of `active`
        template:
          format: "{{ prev.first_name | lower }}-{{ final.active }}-{{ _1 }}"
          rules:
            # random email
            - email: {}
      last_name:
        # using of original value (keep the first letter of the last name)
        template:
          format: "{{ _0 | truncate(length=1) }}"
    rule_order:
      - active
      - email

  - name: film
    rules:
      fulltext:
        # no transformation
        none: ~
      length:
        # random number
        random_num:
          min: 50
          max: 200
      rating:
        pipeline:
          # using pipelines
          pipes:
            - template:
                format: "r"
            - capitalize: ~

  - name: film_actor
    rules: {}
    query:
      # not dumping the actor with id = 132 (Adam Hopper)
      dump_condition: "actor_id <> 132"

  - name: payment
    rules:
      amount:
        # using the value from globals
        template:
          format: "{{ prev.amount | float * payment_k }}"

  - name: staff
    rules:
      email:
        email: {}
      username:
        template:
          # using the values from globals and template variables
          format: "{{ global_value }}.{{ template_var }}.{{ _1 }}"
          rules:
            # random number
            - random_num:
                min: 100
                max: 999
          variables:
            template_var: "tv456"
      password:
        # random hex token
        hex_token:
          len: 40

default:
  locale: EN

# some global variables (they are available in templates)
globals:
  global_value: "gv123"
  payment_k: 1.73
```

The config file contains following sections:

| Section                     | Mandatory | YAML type  | Description
|---                          |---        |---         |---
| [tables](#tables)           | yes       | list       | A list of anonymized tables
| [table_order](#table_order) | no        | list       | An order of table dumping
| [default](#default)         | no        | dictionary | Default values for different anonymization rules
| [filter](#filter)           | no        | dictionary | A filter for tables schema and data (what to skip when dumping)
| [asserts](#asserts)         | no        | list       | SQL assertions executed before dumping starts
| [globals](#globals)         | no        | dictionary | Some global values (they are available in anonymization templates)

## tables

The `tables` section is a list of anonymized [tables](#table).
This is a main element of the config.

Example (there are anonymization rules for two database tables: `actor` and `address`):

```yaml
tables:
  - name: actor
    rules:
      first_name:
        # random name
        first_name: {}
      last_name:
        # random surname
        last_name: {}
      last_update:
        # random date
        datetime:
          from: 1990-01-01T00:00:00+00:00
          to: 2010-12-31T00:00:00+00:00
    query:
      # keeping data of the actor Jane Jackman unanonymized
      transform_condition: "NOT (first_name = 'Jane' AND last_name = 'Jackman')"
      # not dumping the actor with actor_id = 132 (Adam Hopper)
      dump_condition: "actor_id <> 132"

  - name: address
    rules:
      address:
        # using template
        template:
          # using transformed (anonymized) value of district
          format: "{{ final.district }}, {{ _1 }}, {{ _2 }}"
          rules:
            # random street name
            - street_name: {}
            # random building number
            - building_number: {}
```

### table

| Section                   | Mandatory | YAML type  | Description
|---                        |---        |---         |---
| `name`                    | no*       | text       | The table name (exact or wildcard pattern)
| `names`                   | no*       | list       | A list of table name patterns
| [rules](#rules)           | yes       | dictionary | Anonymization rules for this table (the column names are the dictionary keys)
| [rule_order](#rule_order) | no        | list       | An order of rule execution
| [query](#query)           | no        | dictionary | Conditions for SQL queries for dumping data
| [asserts](#asserts)       | no        | list       | SQL assertions executed for this table before dumping starts

\* Either `name` or `names` should be provided.

You can use table names with schema (e.g. `public.users`) or without it (just `users`). In the latter case, this means
that the rules will be applied to the `users` table in any schema.

#### Wildcard patterns in table names

Table names (in both `name` and `names`) support wildcard patterns:

* `*` matches arbitrary many (including zero) occurrences of any character
* `?` matches exactly one occurrence of any character

##### Wildcard table names

Use wildcards in `name` to match multiple tables:

```yaml
tables:
  # Match all tables in the public schema
  - name: "public.*"
    rules:
      email:
        email: {}
```

##### The `names` field

Use `names` to apply the same rules to tables matching any of several patterns:

```yaml
tables:
  # Apply to all tables in schemas A and B
  - names: ["A.*", "B.*"]
    rules:
      email:
        email: {}
```

When a table is matched via a wildcard pattern, exact column names that don't exist in that table
are silently skipped — the same rule set may apply to tables with different schemas.

When a table is matched exactly (`name: users` or a non-wildcard entry in `names`), exact column
names that don't exist produce an error, catching typos.

##### Precedence

When multiple table entries could match, the most specific one wins:

1. **Exact table name** (`public.users`) always takes priority over wildcards
2. **First matching wildcard entry** wins when multiple wildcard entries could match (config order matters)

Example:

```yaml
tables:
  # Wildcard: applies to all public tables
  - name: "public.*"
    rules:
      email:
        email: {}

  # Exact: overrides the wildcard for this specific table
  - name: public.users
    rules:
      email:
        email:
          uniq: true
```

In this example, `public.users` uses the exact entry (with `uniq: true`), while all other
`public.*` tables use the wildcard entry.

#### rules

Anonymization rules (we call them `transformers`) for the table columns.

Dictionary keys are the exact column names.
Each value contains an anonymizing configuration for column (a name of transformer - an address, a company name, a person name, 
some template, etc, with its options).

| Rule (transformer)             | Description                                                                   |
|--------------------------------|------------------------------------------------------------------------------ |
| `email`                        | Emails with different options                                                 |
| `ip`                           | IP addresses. Supports IPv4 and IPv6                                          |
| `words`                        | Lorem words with different length                                             |
| `first_name`                   | First name generator                                                          |
| `last_name`                    | Last name generator                                                           |
| `city`                         | City names generator                                                          |
| `phone`                        | Generate random phone with different `format`                                 |
| `pipeline`                     | Use pipeline to generate more complicated values                              |
| `null`                         | Sets the field value to NULL                                                  |
| `capitalize`                   | Like filter, it capitalizes input value                                       |
| `template`                     | Template engine for generate random text with included rules                  |
| `digit`                        | Random digit (in range `0..9`), localized                                               |
| `random_num`                | Random number with `min` and `max` options                                    |
| `password`                     | Password with different length options<br> (supports `max` and `min` options) |
| `datetime`                     | Make DateTime strings with options (`from` and `to`)                          |
| more than 70 rules in total... |                                                                               |

For the complete list of rules please refer [this document](transformers.md).

**Some transformer examples:**

##### first_name

It gets a person first name.

Examples:

The default:

```yaml
rules:
  field_name:
    first_name: {}
```

You can configure locale:

```yaml
rules:
  field_name:
    first_name:
      locale: RU
```

##### phone

It gets a random phone number.

Examples:

The default:

```yaml
rules:
  field_name:
    phone: {}      
```

You can specify the phone format:

```yaml
rules:
  field_name:
    phone:
      format: "+7^#########"
```

where: 
* `#` - any digit from 0 to 9
* `^` - any digit from 1 to 9
  
Also, you can use any other symbols in format: `^##-00-### (##-##)`.

The default format is `+###########`.

If you want to generate unique phone numbers for this database column, use the `uniq` option:

```yaml
rules:
  field_name:
    phone:
      uniq: true
```

The transformer will collect information about generated numbers and check their uniqueness.
If such a number already exists in the list, then the transformer will try to generate the value again.
The number of attempts is limited by the number of available invariants based on the format.

##### random_num

Gets a random number.

Examples:

The default:

```yaml
rules:
  field_name:
    random_num: {}
```

You can specify a range (one border or both):

```yaml
rules:
  field_name:
    random_num:
      min: 10
      max: 20
```

The default range is from `0` to `2^64 - 1` (for 64-bit application binary).

If you want to generate unique numbers, use this option:

```yaml
  rules:
    field_name:
      random_num:
        uniq: true
```

The transformer will collect information about generated numbers and check their uniqueness.
If such a number already exists in the list, then the transformer will try to generate the value again.
You can limit the number of tries (the default is 3):

```yaml
rules:
  field_name:
    random_num:
      uniq:
        required: true
        try_count: 5
```

##### template

This is the most sophisticated and flexible transformer.

It uses the [Tera](https://tera.netlify.app) template engine
(inspired by [Jinja2](https://jinja.palletsprojects.com)).

Specification:

| Section                   | Mandatory | YAML type  | Description
|---                        |---        |---         |---
| `format`                  | yes       | text       | The template for generated value
| `rules`                   | no        | list       | Nested rules (transformers). You can use them in the template
| `variables`               | no        | dictionary | Template variables

Examples:

```yaml
rules:
  field_name:
    template:
      format: "Hello, {{name}}! {{_1}}:{{_0 | upper}}"
      rules:
        - email: {}    
      variables:
        name: Alex
```

where:
* `_0` - transformed value (original);
* `_1`, `_2`, ... `_N` - nested rules by index (started from 1). You can use any transformer (including templates); 
* `name` - the named variable from the `variables` section.

It will generate something like `Hello, Alex! some-fake-email@gmail.com:ORIGINALVALUE`.

You can use any filter or markup from the Tera template engine.

Also, you can use the [global](#globals) variables in templates.

You can reference values of other row fields in templates.
Use the `prev` special variable for original values and the `final` special variable - for anonymized:

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

You must specify the order of rule execution when using `final` with [rule_order](#rule_order).
All rules not listed will be placed at the beginning (i.e., you must list only rules with `final`).

#### rule_order

A list of columns that will be processed in the specified order (after all columns that are not in the list). 
The order of execution for other columns is not guaranteed. 

Look at this table configuration example:

```yaml
name: customer
rules:
  active:
    # using anonymized `activebool` value
    template:
      format: "{% if final.activebool == 'TRUE' %}1{% else %}0{% endif %}"
  activebool:
    # the probability of `true` is 80%
    boolean:
      ratio: 80
  create_date:
    datetime:
      from: 2000-01-01T00:00:00+00:00
      to: 2020-12-31T00:00:00+00:00
  email:
    # using the original first name value in the anonymized email
    # also using the anonymized value of `active`
    template:
      format: "{{ prev.first_name | lower }}-{{ final.active }}-{{ _1 }}"
      rules:
        # random email
        - email: {}
  last_name:
    # using of original value (keep the first letter of the last name)
    template:
      format: "{{ _0 | truncate(length=1) }}"
rule_order:
  - active
  - email
```

The order of column processing will be as follows:

1. `activebool`, `create_date`, `last_name` (the exact order is not guaranteed)
2. `active`
3. `email`

_You only need the `rule_order` section when using the `template` transformer with the `final` special template variable._ 

For additional information please refer to the [template](transformers.md#template) transformer documentation.

#### query

| Section               | Mandatory | YAML type | Description
|---                    |---        |---        |---
| `dump_condition`      | no        | text      | SQL `WHERE` statement for dumped data
| `limit`               | no        | integer   | SQL `LIMIT` for dumped data
| `transform_condition` | no        | text      | SQL `WHERE` statement for anonymizing data

You can specify conditions (SQL `WHERE` statement) and limit for dumped data from the table:

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

If you don't need data from a particular table at all, please refer to the [filter](#filter) section.

## asserts

Assertions are SQL checks executed against the source database before dumping starts. You can define
them globally or inside a specific table. Both forms use the same YAML schema.

See also the complete example config: [`docs/examples/asserts.yml`](examples/asserts.yml).

```yaml
asserts:
  - name: no_orphan_payments
    sql: |
      select p.id
      from payments p
      left join users u on u.id = p.user_id
      where u.id is null
    expect: no_rows
    message: "payments must reference existing users"
    severity: error

tables:
  - name: users
    rules: {}
    asserts:
      - name: email_not_null
        sql: |
          select count(*) from users where email is null
        expect:
          eq: 0
```

### assert

| Section    | Mandatory | YAML type        | Description
|---         |---        |---               |---
| `name`     | yes       | text             | Assertion name shown in errors
| `sql`      | yes       | text             | SQL query executed before dump
| `expect`   | yes       | text/dictionary  | Expected result (`no_rows`, `rows_exist`, or scalar comparisons)
| `message`  | no        | text             | Extra context added to failure output
| `severity` | no        | text             | `error` stops dumping, `warn` logs a warning

`expect: no_rows` means the SQL query must return zero rows.

```yaml
asserts:
  - name: no_duplicate_emails
    sql: |
      select email
      from users
      group by email
      having count(*) > 1
    expect: no_rows
```

`expect.eq` is intended for scalar queries that return exactly one value.

```yaml
asserts:
  - name: active_users_present
    sql: |
      select count(*) from users where active = true
    expect:
      eq: 42
```

`expect: rows_exist` means the SQL query must return at least one row.

```yaml
asserts:
  - name: users_exist
    sql: |
      select 1 from users limit 1
    expect: rows_exist
```

Scalar comparisons may be combined, and all of them must pass (`AND` semantics).

```yaml
asserts:
  - name: pending_jobs_in_range
    sql: |
      select count(*) from jobs where state = 'pending'
    expect:
      gt: 0
      lte: 100
```

Supported scalar operators:

- `eq`
- `not_eq`
- `gt`
- `gte`
- `lt`
- `lte`

Scalar comparisons require the SQL query to return exactly one scalar value. Numeric operators
(`gt`, `gte`, `lt`, `lte`) require a numeric result.

Recommended style:

- use `expect: no_rows` and `expect: rows_exist` only for row-based checks
- use dictionary form for scalar checks, even when there is only one operator
- combine scalar operators only when all conditions must pass (`AND`)

Global assertions run first. Table assertions run after them in the same order as tables are defined
in the config.

## table_order

A list of tables that will be dumped in the specified order (after all tables that are not in the list).
The order of execution for other tables depends on foreign keys.

Look at this configuration example:

```yaml
tables:
  - name: "table1"
    rules: {}
  - name: "table2"
    rules: {}
  - name: "table3"
    rules: {}    
table_order:
  - "table1"
  - "table2"
```

The order of table dumping will be as follows:

1. `table3`
2. `table1`
3. `table2`

You may need this section when using the built-in key-value store in the `template` transformer for sharing data between
tables.

For additional information please refer to the [template](transformers.md#template) transformer documentation.

## default

| Section          | Mandatory | YAML type | Description
|---               |---        |---        |---
| `locale`         | no        | text      | The default locale for transformers
| `preserve_null`  | no        | boolean   | When `true`, NULL values (`\N`) are preserved instead of transformed (default: `false`)

Supported locales are `EN` (the default one), `ZH_TW` (traditional chinese) and `RU` (translation in progress).
We plan to support more locales in the future.

You can override the locale for each transformer (rule) in its options. Some transformers are not affected by locale.

By default, NULL values (represented as `\N` in PostgreSQL COPY format) are passed to transformers
and replaced with generated data. Set `preserve_null: true` to keep NULLs as-is globally.

Example:

```yaml
default:
  locale: RU
  preserve_null: true
```

## filter

You can specify which tables you choose (whitelisting) or ignore (blacklisting) to dump.

You must use the full table names here (with schema). 

You can use wildcards: 

* `?` matches exactly one occurrence of any character;
* `*` matches arbitrary many (including zero) occurrences of any character.


### Examples

For dumping only `public.markets` and `public.users` data:

```yaml
filter:
  only:
    - public.markets
    - public.users
```

For ignoring these tables and dump data from others:

```yaml
filter:
  except:
    - public.markets
    - public.users
```

You can also specify data and schema filters separately.

This is equivalent to the previous example:

```yaml
filter:
  data:
    except:
      - public.markets
      - public.users
```

For skipping schema and data from other tables:

```yaml
filter:
  schema:
    only:
      - public.markets
      - public.users
```

For skipping schema for `markets` table and dumping data only from `users` table:

```yaml
filter:
  data:
    only:
      - public.users
  schema:
    except:
      - public.markets
```

For skipping schema and data from all tables in the schema `other` (you should use the quotes):

```yaml
filter:
  schema:
    except:
      - "other.*"
```

For dumping data only from `public.table1`, `public.table2`, `public.table3`, etc:

```yaml
filter:
  - "public.table?"
```

If you need only a subset of the data, please refer to the [query](#query) section.

## templates
You can specify some templates in config to reuse them in you [template](transformers.md#template) rules.
There are different kinds of templates:

- `raw` templates is named templates which may be imported or included by name into your field template, you can use macros to extend complex template.
- `files` templates is array of paths to files with template context.

```yaml
tables:
  - name: some_page
    rules:
      some_column:
        template:
          format: >
            {% import "base" as macros -%}
            {{ macros::decrement(n=10) }}
templates:
  raw:
    base: >
      {% macro decrement(n) -%}
      {% if n > 1 %}{{ n }}-{{ self::decrement(n=n-1) }}{% else %}1{% endif -%}
      {% endmacro decrement -%}"#;
  files:
    - ./templates/button.html
```

## globals

You can specify global variables available in all [template](transformers.md#template) rules.

```yaml
tables:
  - name: payment
    rules:
      amount:
      # using the value from globals
      template:
        format: "{{ prev.amount | float * payment_k }}"

  - name: staff
    rules:
      username:
        template:
          # using the value from globals
          format: "{{ global_value }}.{{ _1 }}"
            rules:
              # random number
              - random_num:
                  min: 100
                  max: 999

# global variables (they are available in templates)
globals:
  global_value: "gv123"
  payment_k: 1.73
```
