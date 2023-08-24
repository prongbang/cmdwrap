create:
	cargo shuttle init --template warp gateway

run:
	cargo shuttle run

deploy:
	cargo shuttle deploy --allow-dirty

start:
	cargo shuttle project start

test:
	curl -X GET http://127.0.0.1:8000/get?id=1
	curl -X POST http://127.0.0.1:8000/post -H "Content-Type: application/json" -d '{"key": "value"}'
	curl -X PUT http://127.0.0.1:8000/put?id=1 -H "Content-Type: application/json" -d '{"key": "value"}'
	curl -X PATCH http://127.0.0.1:8000/patch?id=1 -H "Content-Type: application/json" -d '{"key": "value"}'
	curl -X DELETE http://127.0.0.1:8000/delete?id=1
	curl -X OPTIONS http://127.0.0.1:8000/options?id=1