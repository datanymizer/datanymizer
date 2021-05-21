+++
title = "Internet transformers"
description = "Transformers to use with internet topics"
date = 2021-05-01T08:00:00+00:00
updated = 2021-05-01T08:00:00+00:00
draft = false
weight = 10
sort_by = "weight"
template = "docs/page.html"

[extra]
toc = true
top = false
+++

## Email

Transformer generates random emails
```yaml
#...
rules:
  field_name:
    email:
      kind: Safe
```

You can user different <code>kind</code> for email transformer:

* <code>Free</code> - Used by default.
* <code>FreeProvider</code> - Only for "gmail.com", "yahoo.com", "hotmail.com" providers
* <code>Safe</code> - Generates only for .com, .net, .org domains

## IP address

To generate IP address in <code>IPv4</code> format:

```yaml
#...
rules:
  field_name:
    ip: {} #generates V4 format by default
```

You can use <code>IPv6</code> format like:

```yaml
#...
rules:
  field_name:
    ip:
      kind: V6
```

You can use <code>kind</code> value from list:

* <code>V4</code> - It generates IPv4 format address.
* <code>V6</code> - It generates IPv6 format address.

## Password

It generates random password values.

```yaml
#...
rules:
  field_name:
    password: {}
```

You can set minimum and maximum string length (both or one of):
```yaml
#...
rules:
  field_name:
    password:
      min: 5
      max: 10
```
