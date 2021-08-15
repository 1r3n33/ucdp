const Ucdp = artifacts.require("Ucdp");

module.exports = function (deployer) {
  deployer.deploy(Ucdp);
};
