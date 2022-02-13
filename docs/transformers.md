# Transformers

We call the anonymization rules for the table columns `transformers`.

You should use them in the [rules](config.md#rules) sections of the configuration file.

Example:

```yaml
rules:
  # `field_name` is a name of database column  
  field_name:
    # gets a person full name  
    person_name: {}
```

In other examples we will omit the `rules.field_name` part.

This document contains the full list of transformers (grouped by categories).

## Common configuration options 

Many of transformers don't need any configuration. You can use them just like this:

```yaml
# gets a person last name  
last_name: {}
```

If there are no specific notes in the documentation for some transformer,
then there are no configuration options for it.

### Locales

Also, many of them support locale configuration:

```yaml
# gets a person last name  
last_name:
  locale: RU
```

Please refer [here](config.md#default) for the list of available locales.

In this document we mark such transformers with the globe symbol 🌐.  

For some transformers, specifying the locale now may not have any practical effect
(but it may have an effect in the future).

### Uniqueness

You can specify that result values must be unique (they are not unique by default).
You can use short or full syntax.

Short:
```yaml
email:
  uniq: true
```

Full:
```yaml
email:
  uniq:
    required: true
    try_count: 5
```

Uniqueness is ensured by re-generating values when they are same.

You can customize the number of attempts with `try_count` (this is an optional field, the default
number of tries depends on the rule, for some rules it can be guessed automatically).

Currently, uniqueness is supported by: [email](#email), [ip](#ip), [phone](#phone), 
[random_num](#random_num).

In the future, we plan to add support for the uniqueness option for all transformers.  

## Available transformers

### Basic types

#### boolean

Gets a boolean value (TRUE/FALSE), with a given probability.

Examples:

The default:

```yaml
boolean: {}
```

You can specify the probability of TRUE value:

```yaml
boolean:
  # 40% for the TRUE and 60% for the FALSE  
  ratio: 40
```

#### datetime

Generates random dates in the specified interval (granularity is a second).

Examples:

The default:

```yaml
datetime: {}
```

You can specify a range:

```yaml
datetime:
  from: "1990-01-01T00:00:00+00:00"
  to: "2010-12-31T00:00:00+00:00"
```

Also, you can specify the datetime format.

```yaml
datetime:
  format: "%Y-%m-%d"
```

For the bounds (`from`/`to`) you should use the RFC 3339 format (`%Y-%m-%dT%H:%M:%S%.f%:z`).

The default output format is also RFC 3339. 
You don't need to change the format when using this transformer with datetime SQL fields.

[Here](https://docs.rs/chrono/0.4.19/chrono/format/strftime/index.html) you can look at the available formatting 
patterns.

Notes:

* `%C`, `%Z` and `%s` are not supported.
* `%.f` works like `%.9f` (always 9 digits). The behaviour of the `%+` pattern is the same in this regard.
* Patterns (e.g. `%x`, `%X`, `%c`) are not localized.
* Modifiers `_`, `-`, `0` are not supported yet (you can make a feature request).

These are due to the fact that we removed the dependency on the [chrono](https://crates.io/crates/chrono) crate and use 
the [time](https://crates.io/crates/time) crate directly (because of 
[security issue](https://github.com/chronotope/chrono/pull/578) in `chrono`).

#### random_num

Gets a random number.

Examples:

The default:

```yaml
random_num: {}
```

You can specify a range (one border or both):

```yaml
random_num:
  min: 10
  max: 20
```

The default range is from `0` to `2^64 - 1` (for 64-bit application binary).

If you want to generate unique numbers, use this option:

```yaml
random_num:
  uniq: true
```

The transformer will collect information about generated numbers and check their uniqueness.
If such a number already exists in the list, then the transformer will try to generate the value again.
You can limit the number of tries (the default is `3`):

```yaml
random_num:
  uniq:
    required: true
    try_count: 5
```

## Special

#### capitalize

Capitalize a given value (from the database, or a previous value in the pipeline).

E.g., the `3 short words` value will be transformed to `3 Short Words`.

Example:

```yaml
# You should use ~ (the null value in YAML) for this transformer
capitalize: ~
```

#### none

This transformer just does nothing (some sort of `noop`).

Example:

```yaml
# You should use ~ (the null value in YAML) for this transformer
none: ~
```

#### pipeline

You can use pipelines with complicated rules to generate more difficult values.
You can use any transformers as steps (as well as other pipelines too)

Example:

```yaml
pipeline:
  pipes:
    - email: {}
    - capitalize: ~
```

The pipes will be executed in the order in which they are specified in the config.

#### template

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
template:
  format: "Hello, {{name}}! {{_1}}:{{_0 | upper}}"
  rules:
    - email: {}    
  variables:
    name: Alex
```

where:
* `_0` - original value;
* `_1`, `_2`, ... `_N` - nested rules by index (started from 1). You can use any transformer (including templates);
* `name` - the named variable from the `variables` section.

It will generate something like `Hello, Alex! some-fake-email@gmail.com:ORIGINALVALUE`.

You can use any filter or markup from the Tera template engine.

Also, you can use the [global](config.md#globals) variables in templates.

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

You must specify the order of rule execution when using `final` with [rule_order](config.md#rule_order).
All rules not listed will be placed at the beginning (i.e., you must list only rules with `final`).

Also, we implemented a built-in key-value store that allows information to be exchanged between anonymized rows.

It is available via the custom functions in templates (you can read about Tera functions
[here](https://tera.netlify.app/docs/#functions)).

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

The full list of functions for working with the store:

* `store_read` - returns a value by key, when no such key returns a default value or raises an error 
  if no default value is provided.<br/>
  Arguments: `key`, `default` (the `default` arg is optional).

* `store_write` - stores a value in a key, raises an error when the key is already present.<br/>
  Arguments: `key`, `value`.

* `store_force_write` - like `store_write` but `store_force_write` overrides values without errors.<br/>
  Arguments: `key`, `value`.
  
* `store_inc` - increments a value in a key (in the first time just stores a value). Working only with numbers.<br/>
  Arguments: `key`, `value`. 

Also, you can use the template transformer for returning NULL values for your database.

For PostgreSQL, we must return `\N` from the transformer:

```yaml
template:
  format: '\N'
```

If you need the `\N` literal in your database, please return `\\N` from the transformer.

If you need the `\\N` literal - return `\\\N` and so on.

**Warning!** This behavior can be changed in the future.

## Business

#### company_activity 🌐

Gets a company activity description (e.g., `integrate vertical markets`).

#### company_activity_verb 🌐

Gets a company activity verb.

#### company_activity_adj 🌐

Gets a company activity adjective.

#### company_activity_noun 🌐

Gets a company activity noun.

#### company_motto 🌐

Gets a company motto.

#### company_motto_head 🌐

Gets a head component of a company motto.

#### company_motto_middle 🌐

Gets a middle component of a company motto.

#### company_motto_tail 🌐

Gets a tail component of a company motto.

#### company_name 🌐

Gets a company name.

#### company_name_alt 🌐

Gets a company name (an alternative variant).

#### company_suffix 🌐

Gets a company name suffix (e.g., `Inc.` or `LLC`).

#### industry 🌐

Gets an industry name.

#### profession 🌐

Gets a profession name.


## Currencies

#### currency_code 🌐

Gets a currency code (e.g., `EUR` or `USD`).

#### currency_name 🌐

Gets a currency name.

#### currency_symbol 🌐

Gets a currency symbol.


## Files

#### dir_path 🌐

Gets a file directory path.

#### file_extension 🌐

Gets a file extension.

#### file_name 🌐

Gets a file name.

#### file_path 🌐

Gets a file path.


## Internet and communications

#### domain_suffix 🌐

Gets a domain suffix (e.g., `com`).

#### email

Gets a random email. You can specify a kind. The kind can be `Safe` (it is default) or `Free`.

With the `Safe` kind the transformer generates only emails for example domains (e.g., `some@example.com`).
It is not real email addresses.

With the `Free` kind the transformer generates emails for free email providers (e.g., `some@gmail.com`, 
`some@yahoo.com`, `some@hotmail.com`). 

You can add a random alphanumeric prefix and/or suffix (e.g., `12zsd-some@example.com`, `some-asd1mk@example.com`,
`anahgk-some-a21km@example.com`).
This is useful when you need many unique emails.

Also, you can specify a fixed prefix/suffix (`test-` or `-test`) or use a transformer as a prefix/suffix
(usually, a template).

The default separator for prefixes and suffixes is `-`. You can change it with the `affix_separator` option.

Examples:

The default:

```yaml
email: {}
```

You can specify the kind:

```yaml
email:
  kind: Free
```

With a random prefix:

```yaml
email:
  # prefix length
  prefix: 5
```

With a random suffix:

```yaml
  email:
    # suffix length
    suffix: 5
```

With a fixed prefix:
```yaml
email:
  # prefix content
  prefix: "test"
```

Using a transformer as prefix:

```yaml
email:
  # prefix template
  prefix:
    template:
      format: "........"
      #.......
```

Custom `affix_separator` (`77zsd__some@example.com`):

```yaml
email:
  prefix: 5
  affix_separator: "__"
```

If you want to generate unique emails, use this option:

```yaml
email:
  uniq: true
```

The transformer will collect information about generated emails and check their uniqueness.
If such a email already exists in the list, then the transformer will try to generate the value again.
You can limit the number of tries (the default is `3`):

```yaml
email:
  uniq:
    required: true
    try_count: 5
```

#### free_email_provider 🌐

Gets a free email provider name (e. g., `gmail.com`).

#### ip

Generates an IP address.

You can specify the kind (`V4` or `V6`).

Examples:

The default:

```yaml
ip: {}
```

Default kind is `V4`, you can specify V6:

```yaml
ip:
  kind: V6
```

#### local_cell_phone 🌐

Gets a local cell phone number (for a given locale).

#### local_phone 🌐

Gets a local phone number (for a given locale).

#### mac_address

Gets a MAC address.

#### password

Generates a random password.

You can set minimum and maximum string length.

Examples:

The default:
```yaml
  password: {}
```

With a custom length (the default `min` option is `8` and the `max` option is `20`):

```yaml
password:
  min: 5
  max: 10
```

#### phone

Gets a random phone number.

Examples:

The default:

```yaml
phone: {}      
```

You can specify the phone format:

```yaml
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
phone:
  uniq: true
```

The transformer will collect information about generated numbers and check their uniqueness.
If such a number already exists in the list, then the transformer will try to generate the value again.
The number of attempts is limited by the number of available invariants based on the format.

#### user_agent 🌐

Gets a User-Agent header.

#### username 🌐

Gets a username (login).


## Job

#### job_field 🌐

Gets a job field.

#### job_position 🌐

Gets a job position.

#### job_seniority 🌐

Gets a job seniority (e.g., `Lead`, `Senior` or `Junior`).

#### job_title 🌐

Gets a job title (seniority + field + position).


## Locations

#### building_number 🌐

Gets a building number.

#### city 🌐

Gets a city name.

#### city_prefix 🌐

Gets a city prefix (e.g., `North-` or `East-`).

#### city_suffix 🌐

Gets a city suffix (e.g., `-town`, `-berg` or `-ville`).

#### country_code 🌐

Gets a country code (e.g., `RU`).

#### country_name 🌐

Gets a country name.

#### dwelling_type 🌐

Gets a dwelling unit type (e.g., `Apt.` or `Suit.`).

#### dwelling 🌐

Gets a dwelling unit part of the address (apartment, flat...).

#### latitude

Gets a latitude.

#### longitude

Gets a longitude.

#### post_code 🌐
Gets a post code.

#### state_abbr 🌐

Gets a state (or the equivalent) abbreviation (e.g., `AZ` or `LA`).

#### state_name 🌐

Gets a state (or the equivalent) name.

#### street_name 🌐

Gets a street name.

#### street_suffix 🌐

Gets a street suffix (e.g., `Avenue` or `Highway`).

#### time_zone

Gets a time zone (e.g., `Europe/London`).

#### zip_code 🌐

Gets a zip code.


## People

#### first_name 🌐

Gets a person first name.

#### last_name 🌐

Gets a person last name.

#### middle_name 🌐

Gets a person middle name (a patronymic name, if the locale has such a concept).

#### name_suffix 🌐

Gets a name suffix (e.g., `Jr.`)

#### person_name 🌐

Gets a person name (full).

#### person_name_with_title 🌐

Gets a person name with title.

#### person_title 🌐

Gets a person name title (e.g., `Mr` or `Ms`).


## Text (lorem ipsum or its analog)

#### paragraph 🌐

Gets a "lorem" paragraph (you can specify a count of sentences).

Examples:

The default:

```yaml
paragraph: {}
```

This is equal to:

```yaml
paragraph:
  locale: EN
  # Min count
  min: 2
  # Max count
  max: 5
```

#### paragraphs 🌐

Gets several "lorem" paragraphs (you can specify a count).

Examples:

The default:

```yaml
paragraphs: {}
```

This is equal to:

```yaml
paragraphs:
  locale: EN
  # Min count
  min: 2
  # Max count
  max: 5
```

#### sentence 🌐

Gets a "lorem" sentence (you can specify a count of words).

Examples:

The default:

```yaml
sentence: {}
```

This is equal to:

```yaml
sentence:
  locale: EN
  # Min count
  min: 2
  # Max count
  max: 5
```

#### sentences 🌐

Gets several "lorem" sentences (you can specify a count).

Examples:

The default:

```yaml
sentences: {}
```

This is equal to:

```yaml
sentences:
  locale: EN
  # Min count
  min: 2
  # Max count
  max: 5
```

#### word 🌐

Gets a "lorem" word.

#### words 🌐

Gets several "lorem" words (you can specify a count).

Examples:

The default:

```yaml
words: {}
```

This is equal to:

```yaml
words:
  locale: EN
  # Min count
  min: 2
  # Max count
  max: 5
```

## Tokens

#### base64_token

Generates random Base64 tokens. You can set a token length (default is 32) and a padding (`=` symbols) length.

Examples:

With defaults:
```yaml
#...
base64_token: {}
```

With a custom length:
```yaml
base64_token:
  # the padding is included into the length, so we have 35 symbols and `=`
  len: 36
  pad: 1
```

#### base64url_token

Generates random Base64Url tokens.
You can set a token length (default is 32) and a padding - a number of `%3D` sequences.

Examples:

With defaults:
```yaml
base64_token: {}
```

With a custom length:
```yaml
base64_token:
  # the padding is included into the length, so we have 34 symbols and the padding (`%3D%3D`)
  len: 36
  pad: 2
```

#### hex_token

Generates random hex tokens. You can set a token length (default is 32).

Examples:

The default:

```yaml
  hex_token: {}
```

With a custom length:

```yaml
hex_token:
  len: 128
```


## Others

#### color

Gets a color code (e.g., `#ffffff`).

#### digit 🌐

Gets a localized digit symbol (e.g., `2` or `5` for the English locale).
