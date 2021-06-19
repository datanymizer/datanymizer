# pg_datanymizer

`pg_datanymizer` is the command line application for anonymizing data from PostgreSQL databases.

#### Usage:

```
pg_datanymizer [OPTIONS] <DBNAME>
```

#### FLAGS

| Name              | Description 
|---                |---          
| `--help`          | Prints help information
| `-V`, `--version` | Prints version information

#### OPTIONS

| Name                                                      | Description
|---                                                        |---  
| `-f`, `--file` `<FILE>`                                   | Path to the dump output file, example: `/tmp/dump.sql`
| `-c`, `--config` `<config>`                               | Path to the config file. Default: `./config.yml`
| `--pg_dump` `<pg-dump-location>`                          | Postgres `pg_dump` utility program file location. Default: just `pg_dump`
| `--accept_invalid_certs` `<accept-invalid-certs>`         | Accept or not invalid certificates (e.g., self-signed) when using SSL. Default: `false` 
| `--accept_invalid_hostnames` `<accept-invalid-hostnames>` | Accept or not invalid hostnames when using SSL. Default: `false`
| When `<DBNAME>` is just a database name (not a full url):
| `-h`, `--host` `<host>`                                   | Database server host or a socket directory. Default: `localhost`
| `-W`, `--password` `<password>`                           | User password
| `-p`, `--port` `<port>`                                   | Database server port number. Default: `5432`
| `-U`, `--username` `<username>`                           | Connect as the specified database user

#### ARGS

| Name       | Description
|---         |---  
| `<DBNAME>` | Postgres database URL, e.g. `postgres://postgres:password@localhost:5432/database_name?sslmode=disable` (you can omit some parts), or just a database name, e.g. `my_db`  

