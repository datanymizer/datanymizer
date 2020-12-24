# [Data]nymizer demo

## Get started

1. Install `docker` and `docker-compose` to your development environment.
2. Go to `demo` directory:

``` shell
cd demo
```
and run:

``` shell
make bootstrap
```

It starts docker container with Postgresql, download demo database [PostgreSQL Sample Database](https://www.postgresqltutorial.com/postgresql-sample-database/), unpack it and restore to new `dvdrental` database.

3. For dumping this base, run:

``` shell
make dump
```
It makes new `/tmp/fake_dump.sql` file with fake rows in tables from `dvdrental.yml` configuration.

You can change configuration file and try again.
