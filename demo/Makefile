example_database_url="https://sp.postgresqltutorial.com/wp-content/uploads/2019/05/dvdrental.zip"
tmpfile="/tmp/dump.zip"
pause=5
.SILENT:

build:
	docker-compose build

up:
	echo -e "\n==> Start containers"
	docker-compose up -d
	echo -e "\n==> Pause ${pause}s after starting Postgresql"
	sleep ${pause}

download: clean_file
	echo -e "\n==> Download DVDRENTAL database"
	curl $(example_database_url) -o "$(tmpfile)" && unzip $(tmpfile) -d /tmp

clean_file:
	echo -e "\n==> Clean all temporary files"
	rm -rf $(tmpfile) && rm -rf /tmp/dvdrental.tar

clean_db:
	echo -e "\n==> Clean database if exists"
	docker-compose exec pg_database psql -Upostgres -c "drop database if exists dvdrental;"

create_db: up clean_db
	echo -e "\n==> Create empty database"
	docker-compose exec pg_database psql -Upostgres -c "create database dvdrental;"

restore_db:
	echo -e "\n==> Restore database from dump file"
	docker-compose run --rm -v /tmp/dvdrental.tar:/tmp/dvdrental.tar -e PGPASSWORD=postgres pg_database pg_restore -h pg_database -Upostgres -d dvdrental /tmp/dvdrental.tar

bootstrap: download create_db restore_db clean_file

dump:
	echo -e "\n==> Create dump"
	docker-compose run --rm pg_datanymizer postgres://postgres:postgres@pg_database/dvdrental -c /dvdrental.yml > /tmp/fake_dump.sql
