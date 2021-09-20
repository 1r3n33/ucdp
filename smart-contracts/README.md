> **smart-contracts** is for ucdp ethereum smart contracts

## Build smart-contracts docker image

```console
$ sudo docker build . -t ucdp/smart-contracts --no-cache --pull
```

## Run smart-contracts docker container

```console
$ sudo docker run --rm -t -i -p 8545:8545 ucdp/smart-contracts
```

## Connect docker smart-contracts with truffle

```console
$ truffle console --network docker
truffle(docker)> ucdp = await Ucdp.at("0x81F34DC2C089AF4e11881af04399a7e722feA6F4")
truffle(docker)> await ucdp.registerPartner(web3.utils.fromAscii("partner"), { from: accounts[0] })
truffle(docker)> await ucdp.partners(accounts[0])
```
