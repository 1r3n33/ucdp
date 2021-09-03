> **gateway** is an access point to ucdp services

## Build gateway docker image

```console
$ sudo docker build . -t ucdp/gateway --no-cache --pull
```

## Run a docker gateway with a local kafka

```console
$ sudo docker run --rm -t -i --network=host --env UCDP_STREAM_KAFKA_BROKER=127.0.0.1:9092 ucdp/gateway
```
