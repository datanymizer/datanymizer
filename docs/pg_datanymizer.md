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

| Name                             | Description
|---                               |---  
| `-f`, `--file` `<FILE>`          | Path to the dump output file, example: `/tmp/dump.sql`
| `-c`, `--config` `<config>`      | Path to the config file. Default: `./config.yml`
| `--pg_dump` `<pg-dump-location>` | Postgres `pg_dump` utility program file location. Default: just `pg_dump`

#### ARGS

| Name       | Description
|---         |---  
| `<DBNAME>` | Postgres database URL, e.g. `postgres://postgres:password@localhost:5432/database_name?sslmode=disable`  

