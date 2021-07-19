+++
title = "Quick Start"
description = "How to start a Datanymizer."
date = 2021-05-01T08:20:00+00:00
updated = 2021-05-01T08:20:00+00:00
draft = false
weight = 20
sort_by = "weight"
template = "docs/page.html"

[extra]
lead = ""
toc = true
top = false
+++

## Installation

There are several ways to install <code>pg_datanymizer</code> CLI tool. Choose a more convenient option for you.

### Pre-compiled binary

```bash
# Linux / macOS / Windows (MINGW and etc). Installs it into ./bin/ by default
$ curl -sSfL https://raw.githubusercontent.com/datanymizer/datanymizer/main/cli/pg_datanymizer/install.sh | sh -s

# Or more shorter way
$ curl -sSfL https://git.io/pg_datanymizer | sh -s

# Specify installation directory and version
$ curl -sSfL https://git.io/pg_datanymizer | sh -s -- -b usr/local/bin v0.2.0

# Alpine Linux (wget)
$ wget -q -O - https://git.io/pg_datanymizer | sh -s
```

or run with Docker:

```bash
$ docker run --rm -v `pwd`:/app -w /app datanymizer/pg_datanymizer
```

### Homebrew / Linuxbrew

```bash
# Installs the latest stable release
$ brew install datanymizer/tap/pg_datanymizer

# Builds the latest version from the repository
$ brew install --HEAD datanymizer/tap/pg_datanymizer
```

## How to run
The README contains an example configuration that you can use as a starting point.

Now you can invoke Datanymizer to generate a cleansed dump of your data:

```bash
$ pg_datanymizer -f /tmp/dump.sql -c ./config.yml postgres://postgres:postgres@localhost/test_database
```
