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

| Section             | Mandatory | YAML type  | Description
|---                  |---        |---         |---
| [tables](#tables)   | yes       | list       | A list of anonymized tables  
| [default](#default) | no        | dictionary | Default values for different anonymization rules
| [filter](#filter)   | no        | dictionary | A filter for tables schema and data (what to skip when dumping)
| [globals](#globals) | no        | dictionary | Some global values (they are available in anonymization templates)

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
| `name`                    | yes       | text       | The table name in the database
| [rules](#rules)           | yes       | dictionary | Anonymization rules for this table (the column names are the dictionary keys)
| [rule_order](#rule_order) | no        | list       | An order of rule execution
| [query](#query)           | no        | dictionary | Conditions for SQL queries for dumping data 

#### rules

Anonymization rules (we call them `transformers`) for the table columns.

Dictionary keys are the column names.
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

For additional information please refer to the [template](#template) transformer documentation.

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

## default

| Section       | Mandatory | YAML type | Description
|---            |---        |---        |---
| `locale`      | no        | text      | The default locale for transformers

Supported locales are `EN` (the default one), `ZH_TW` (traditional chinese) and `RU` (translation in progress).
We plan to support more locales in the future.

You can override the locale for each transformer (rule) in its options. Some transformers are not affected by locale.

Example:

```yaml
default:
  locale: RU
```

## filter

You can specify which tables you choose (whitelisting) or ignore (blacklisting) to dump.

For dumping only `public.markets` and `public.users` data.

```yaml
filter:
  only:
    - public.markets
    - public.users
```

For ignoring these tables and dump data from others.

```yaml
filter:
  except:
    - public.markets
    - public.users
```

You can also specify data and schema filters separately.

This is equivalent to the previous example.

```yaml
filter:
  data:
    except:
      - public.markets
      - public.users
```

For skipping schema and data from other tables.

```yaml
filter:
  schema:
    only:
      - public.markets
      - public.users
```

For skipping schema for `markets` table and dumping data only from `users` table.

```yaml
filter:
  data:
    only:
      - public.users
  schema:
    except:
      - public.markets
```

If you need only a subset of the data, please refer to the [query](#query) section.

## globals

You can specify global variables available in all [template](#template) rules.

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
