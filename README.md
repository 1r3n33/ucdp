> **ucdp** is a micro customer data platform

## Build ucdp docker image

```console
$ sudo docker build . -t ucdp --no-cache --pull
```

## Run a docker ucdp with a local kafka

```console
$ sudo docker run --rm -t -i --network=host --env UCDP_STREAM_KAFKA_BROKER=127.0.0.1:9092 ucdp
```

## Run with docker-compose

```console
$ sudo docker-compose up
```

## Send an event request to ucdp

```console
curl -X 'POST' 'http://0.0.0.0:8080/v1/events' -H 'accept: application/json' -H 'Content-Type: application/json' -d '[{"name":"abc"}]' -v | jq .
```
