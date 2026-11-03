# Integration tests

To run integration tests for PostgreSQL:

1. Prepare the database server on which you will run the integration tests (for now you must use PostgreSQL 14 for testing).
2. Choose the test database name, e.g. `datanymizer_test`.
   **This database will be recreated during the tests,** as well as several other databases with names starting with
   this name, (e.g. `datanymizer_test_fake_simple`). So don't use the existed database names or its prefixes, please.
3. Add the connection string (database url) to the environment variable `DATANYMIZER_TEST_PG_DB`, e.g.
   `DATANYMIZER_TEST_PG_DB=postgresql://postgres:pgpass@localhost:5432/datanymizer_test`.
4. Run tests with the `pg_db_tests` feature flag enabled:
   `cargo test --features pg_db_tests` or `cargo test --all-features`.

You can change the locations of `pg_dump`, `pg_restore` and `psql` programs with the `DATANYMIZER_TEST_PG_DUMP_PATH`,
`DATANYMIZER_TEST_PG_RESTORE_PATH` and `DATANYMIZER_TEST_PSQL_PATH` environment variables (the default ones are just
`pg_dump`, `pg_restore` and `psql`).

To run integration tests for MySQL (works with MySQL 8.x and MariaDB):

1. Prepare the database server, e.g. `docker run -d -p 127.0.0.1:3307:3306 -e MYSQL_ROOT_PASSWORD=pass mysql:8.4`.
2. Choose the test database name, e.g. `datanymizer_test`.
   **This database will be recreated during the tests,** as well as databases with names starting with
   this name (e.g. `datanymizer_test_dump`).
3. Add the database url to the environment variable `DATANYMIZER_TEST_MYSQL_DB`, e.g.
   `DATANYMIZER_TEST_MYSQL_DB=mysql://root:pass@127.0.0.1:3307/datanymizer_test`.
4. Run tests with the `mysql_db_tests` feature flag enabled:
   `cargo test --features mysql_db_tests` or `cargo test --all-features`.

You can change the locations of `mysqldump` and `mysql` programs with the `DATANYMIZER_TEST_MYSQLDUMP_PATH` and
`DATANYMIZER_TEST_MYSQL_PATH` environment variables (the default ones are just `mysqldump` and `mysql`).
