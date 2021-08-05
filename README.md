> **ucdp** is a micro customer data platform

## Run a docker ucdp with a local kafka

```console
$ sudo docker build . -t ucdp
```

```console
$ sudo docker run --rm -t -i --network=host ucdp
```

## Run a local ucdp with a docker-compose kafka 

```console
$ sudo docker-compose up
```

```console
$ UCDP_STREAM_KAFKA_BROKER=127.0.0.1:29092 cargo run
```
