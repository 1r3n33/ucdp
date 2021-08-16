> **gateway** is an access point to ucdp services

## Build ucdp docker image

```console
$ sudo docker build . -t ucdp --no-cache --pull
```

## Run a docker ucdp with a local kafka

```console
$ sudo docker run --rm -t -i --network=host --env UCDP_STREAM_KAFKA_BROKER=127.0.0.1:9092 ucdp
```
