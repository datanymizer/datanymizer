# pg_datanymizer

`pg_datanymizer` is the command line application for anonymizing data from PostgreSQL databases.

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
