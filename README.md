> **ucdp** is a micro customer data platform

## Run with docker-compose

```console
$ sudo docker-compose up
```

## Send an event request to ucdp

```console
$ curl 'http://0.0.0.0:8080/v1/events' -H 'accept: application/json' -H 'Content-Type: application/json' -d '[{"name":"abc"}]' -v | jq .
```
