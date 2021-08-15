const Ucdp = artifacts.require("Ucdp");

contract("Ucdp", async (accounts) => {
  it("should be deployed correctly", async () => {
    const ucdp = await Ucdp.deployed();
    const partner = await ucdp.partners(web3.utils.padLeft(0x123, 40));
    assert.equal(web3.utils.toUtf8(partner.name), "partner");
    assert.equal(partner.enabled, true);
  });
});
