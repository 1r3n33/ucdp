FROM ubuntu

RUN apt-get update
RUN apt-get install -y curl

# Node.js v12.x
RUN curl -fsSL https://deb.nodesource.com/setup_12.x | bash -
RUN apt-get install -y nodejs

RUN mkdir -p /ucdp/smart-contracts

# Copy resources for build, test and run
COPY truffle-config.js /ucdp/smart-contracts
COPY contracts /ucdp/smart-contracts/contracts
COPY migrations /ucdp/smart-contracts/migrations
COPY test /ucdp/smart-contracts/test
COPY scripts /ucdp/smart-contracts/scripts

# Execute test
WORKDIR /ucdp/smart-contracts
RUN npm install truffle
RUN npx truffle test --compile-all

ENTRYPOINT [ "/ucdp/smart-contracts/scripts/docker-entrypoint.sh" ]
