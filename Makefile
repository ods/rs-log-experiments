.PHONY: start-graylog setup-sentry start-sentry

# After the 1st start of Graylog:
#   * Go to http://localhost:9000/system/inputs
#   * Choose "GELF TCP input" in "Select input" drop-down menu
#   * Click on "Launch new input" button
#   * Edit the form (e.g. by checking "Global" and filling in "Title" input)
#   * Click save

start-graylog:
	docker-compose up -d graylog

# Before the 1st start of Sentry:
#   * Execute `make setup-sentry`

setup-sentry:
	docker-compose up -d pg
	sleep 3
	docker-compose exec pg createdb sentry
	docker-compose run --rm sentry upgrade --noinput
	docker-compose run --rm sentry createuser --no-input --email sentry --password sentry
	docker-compose exec pg psql sentry -c "UPDATE sentry_projectkey SET public_key='185b7a7e069f4ef0983c2467e79683b1', secret_key='064ae6a0c80646478544516572519d6d'"

start-sentry:
	docker-compose run --rm -d sentry run worker
	docker-compose up -d sentry
