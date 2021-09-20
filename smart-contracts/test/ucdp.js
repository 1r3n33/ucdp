const Ucdp = artifacts.require("Ucdp");

contract("Ucdp", async (accounts) => {
  it("should be deployed correctly", async () => {
    const ucdp = await Ucdp.deployed();
    const partner = await ucdp.partners(web3.utils.padLeft(0x123, 40));
    assert.equal(web3.utils.toUtf8(partner.name), "partner");
    assert.equal(partner.enabled, true);
    assert.equal(partner.registered, true);
  });

  it("should register a new partner", async () => {
    const ucdp = await Ucdp.deployed();
    await ucdp.registerPartner(web3.utils.fromAscii("new partner"), {
      from: accounts[0],
    });
    const partner = await ucdp.partners(accounts[0]);
    assert.equal(web3.utils.toUtf8(partner.name), "new partner");
    assert.equal(partner.enabled, true);
    assert.equal(partner.registered, true);
  });

  it("should not register a partner twice", async () => {
    const ucdp = await Ucdp.deployed();

    let hasRaisedException = false;
    try {
      await ucdp.registerPartner(web3.utils.fromAscii("not a new partner"), {
        from: accounts[0],
      });
    } catch (exception) {
      hasRaisedException = true;
      assert.equal(
        exception.message.includes("Sender already registered"),
        true
      );
    }
    assert.equal(hasRaisedException, true);
  });
});
