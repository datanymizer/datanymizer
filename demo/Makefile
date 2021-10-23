example_database_url="https://www.postgresqltutorial.com/wp-content/uploads/2019/05/dvdrental.zip"
tmpfile="/tmp/dump.zip"
dump_file="/tmp/fake_dump.sql"
count_query="SELECT COUNT(*) FROM actor;"
data_query="SELECT first_name, last_name FROM actor LIMIT 1;"
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
	docker-compose exec -T pg_database psql -Upostgres -c "drop database if exists dvdrental;"

clean_fake_db:
	echo -e "\n==> Clean database for fake data if exists"
	docker-compose exec -T pg_database psql -Upostgres -c "drop database if exists dvdrental_fake;"

create_db: up clean_db
	echo -e "\n==> Create an empty database"
	docker-compose exec -T pg_database psql -Upostgres -c "create database dvdrental;"

create_fake_db: clean_fake_db
	echo -e "\n==> Create an empty database for fake data"
	docker-compose exec -T pg_database psql -Upostgres -c "create database dvdrental_fake;"

restore_db:
	echo -e "\n==> Restore database from dump file"
	docker-compose run --rm -v /tmp/dvdrental.tar:/tmp/dvdrental.tar -e PGPASSWORD=postgres pg_database pg_restore -h pg_database -Upostgres -d dvdrental /tmp/dvdrental.tar

bootstrap: download create_db restore_db clean_file

dump:
	echo -e "\n==> Create the dump"
	docker-compose run --rm pg_datanymizer postgres://postgres:postgres@pg_database/dvdrental -c /dvdrental.yml > $(dump_file)

restore_dump: create_fake_db
	echo -e "\n==> Restore the dump to the database for fake data"
	cat $(dump_file) | docker-compose exec -T pg_database psql -Upostgres -d dvdrental_fake -v "ON_ERROR_STOP=1"

check_data:
	echo -e "\n==> Check data in the both databases"
	( \
	  [ \
	    `docker-compose exec -T pg_database psql -At -Upostgres -d dvdrental -c $(count_query)` = \
	    `docker-compose exec -T pg_database psql -At -Upostgres -d dvdrental_fake -c $(count_query)` \
	  ] && echo "The row counts are equal" \
	) || (echo "The row counts are different" && exit 1)
	( \
	  [ \
	      `docker-compose exec -T pg_database psql -At -Upostgres -d dvdrental -c $(data_query)` != \
	      `docker-compose exec -T pg_database psql -At -Upostgres -d dvdrental_fake -c $(data_query)` \
	  ] && echo "The data is different" \
	) || (echo "The data is the same" && exit 1)
