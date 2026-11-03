# pg_datanymizer

`pg_datanymizer` is the command line application for anonymizing data from PostgreSQL databases.

If the configuration contains SQL assertions, `pg_datanymizer` executes them before writing the
dump. Failed assertions with `severity: error` stop the process; `severity: warn` prints warnings
and continues.

#### Usage:

```
pg_datanymizer [OPTIONS] <DBNAME> [PG_DUMP_ARGS]...
```

#### FLAGS

| Name                         | Description 
|---                           |---          
| `--accept_invalid_certs`     | Accept invalid certificates (e.g., self-signed) when using SSL
| `--accept_invalid_hostnames` | Accept invalid hostnames when using SSL
| `--help`                     | Prints help information
| `-V`, `--version`            | Prints version information
| `-v`, `--verbose`            | Turn on verbose logging to show more information about errors

#### OPTIONS

| Name                                      | Description
|---                                        |---  
| `-f`, `--file` `<FILE>`                   | Path to the dump output file, example: `/tmp/dump.sql`
| `-c`, `--config` `<config>`               | Path to the config file. Default: `./config.yml`
| `--pg_dump` `<pg-dump-location>`          | Postgres `pg_dump` utility program file location. Default: just `pg_dump`
| `--mysqldump` `<mysqldump-location>`      | `mysqldump` utility program file location (for `mysql://` URLs). Default: just `mysqldump`
| `--dump-transaction` `<dump-transaction>` | Using a transaction when dumping data, you can specify the isolation level. Possible values: `NoTransaction`, `ReadUncommitted`, `ReadCommitted`, `RepeatableRead`, `Serializable`. Default: `ReadCommitted`.
| When `<DBNAME>` is just a database name (not a full url):
| `-h`, `--host` `<host>`                   | Database server host or a socket directory. Default: `localhost`
| `-W`, `--password` `<password>`           | User password
| `-p`, `--port` `<port>`                   | Database server port number. Default: `5432`
| `-U`, `--username` `<username>`           | Connect as the specified database user

#### ARGS

| Name             | Description
|---               |---  
| `<DBNAME>`       | Postgres database URL, e.g. `postgres://postgres:password@localhost:5432/database_name?sslmode=disable` (you can omit some parts), or just a database name, e.g. `my_db`  
| `<PG_DUMP_ARGS>` | The remaining arguments are passed directly to `pg_dump` calls. You should add `--` before `<DBNAME>` in such cases

## MySQL / MariaDB

Pass a `mysql://` URL instead of a Postgres one. The same config file format works:

```shell
pg_datanymizer -c config.yml -f dump.sql --dump-transaction RepeatableRead \
  "mysql://user:password@127.0.0.1:3306/app"
```

How the dump is built:

* The schema (tables, views) comes from `mysqldump --no-data --skip-triggers`; triggers are dumped after the data
  (so they don't fire during restore). Extra args after `--` go to every `mysqldump` call, e.g. `-- --routines --events`.
* Data is read with `SELECT` and written as multi-row `INSERT`s (unqualified, so the dump can be restored into
  a database with another name), with `FOREIGN_KEY_CHECKS=0` and `TIME_ZONE='+00:00'`.
* The "schema" part of a table name is the database name: filters use `app.users` (rules may use just `users`).
* Generated columns are skipped (they are recalculated on restore); binary columns are written as hex literals.
* `--dump-transaction RepeatableRead` gives a consistent snapshot across all tables (InnoDB).
* TLS is configured with the URL params of the [mysql crate](https://docs.rs/mysql), e.g. `?require_ssl=true`.
* The password is passed to `mysqldump` via the `MYSQL_PWD` environment variable.
* Use the `mysqldump` matching your server: MariaDB's dumps start with a sandbox-mode line that MySQL clients reject.
