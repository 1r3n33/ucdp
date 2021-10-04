const Ucdp = artifacts.require("Ucdp");

contract("Ucdp", async (accounts) => {
  it("should be deployed correctly", async () => {
    const ucdp = await Ucdp.deployed();
    const partnerAddress = web3.utils.padLeft(0x123, 40);
    const userWithPartnerAddress = web3.utils.padLeft(0x456, 40);
    const userWithoutPartnerAddress = web3.utils.padLeft(0x789, 40);

    const partner = await ucdp.partners(partnerAddress);
    assert.equal(web3.utils.toUtf8(partner.name), "partner");
    assert.equal(partner.enabled, true);
    assert.equal(partner.registered, true);

    const userWithPartner = await ucdp.users(userWithPartnerAddress);
    assert.equal(web3.utils.toUtf8(userWithPartner.name), "user auth");
    assert.equal(userWithPartner.registered, true);

    const userWithoutPartner = await ucdp.users(userWithoutPartnerAddress);
    assert.equal(web3.utils.toUtf8(userWithoutPartner.name), "user no-auth");
    assert.equal(userWithoutPartner.registered, true);

    const isPartnerAuthorized1 = await ucdp.authorizedPartnersByUser(
      userWithPartnerAddress,
      partnerAddress
    );
    assert.equal(isPartnerAuthorized1, true);

    const isPartnerAuthorized2 = await ucdp.authorizedPartnersByUser(
      userWithoutPartnerAddress,
      partnerAddress
    );
    assert.equal(isPartnerAuthorized2, false);
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
        exception.message.includes("Sender already registered as a Partner"),
        true
      );
    }
    assert.equal(hasRaisedException, true);
  });

  it("should register a new user", async () => {
    const ucdp = await Ucdp.deployed();
    await ucdp.registerUser(web3.utils.fromAscii("new user"), {
      from: accounts[0],
    });
    const user = await ucdp.users(accounts[0]);
    assert.equal(web3.utils.toUtf8(user.name), "new user");
    assert.equal(user.registered, true);
  });

  it("should not register a user twice", async () => {
    const ucdp = await Ucdp.deployed();

    let hasRaisedException = false;
    try {
      await ucdp.registerUser(web3.utils.fromAscii("not a new user"), {
        from: accounts[0],
      });
    } catch (exception) {
      hasRaisedException = true;
      assert.equal(
        exception.message.includes("Sender already registered as a User"),
        true
      );
    }
    assert.equal(hasRaisedException, true);
  });

  it("should authorize a partner", async () => {
    const ucdp = await Ucdp.deployed();
    // Reuse registred user from account[0]
    const partnerAddress = web3.utils.padLeft(0x123, 40);
    await ucdp.authorizePartner(partnerAddress, { from: accounts[0] });
    const isAuthorized = await ucdp.authorizedPartnersByUser(
      accounts[0],
      partnerAddress
    );
    assert.equal(isAuthorized, true);
  });

  it("should unauthorize a partner", async () => {
    const ucdp = await Ucdp.deployed();
    // Reuse registred user from account[0]
    const partnerAddress = web3.utils.padLeft(0x123, 40);
    await ucdp.unauthorizePartner(partnerAddress, { from: accounts[0] });
    const isAuthorized = await ucdp.authorizedPartnersByUser(
      accounts[0],
      partnerAddress
    );
    assert.equal(isAuthorized, false);
  });

  it("should not authorize an unregistered user", async () => {
    const ucdp = await Ucdp.deployed();

    let hasRaisedException = false;
    try {
      const partnerAddress = web3.utils.padLeft(0x123, 40);
      await ucdp.authorizePartner(partnerAddress, { from: accounts[1] });
    } catch (exception) {
      hasRaisedException = true;
      assert.equal(
        exception.message.includes("Sender must be registered as a User"),
        true
      );
    }
    assert.equal(hasRaisedException, true);
  });

  it("should not authorize an unregistered partner", async () => {
    const ucdp = await Ucdp.deployed();

    let hasRaisedException = false;
    try {
      const unregisteredPartnerAddress = web3.utils.padLeft(0x321, 40);
      await ucdp.authorizePartner(unregisteredPartnerAddress, {
        from: accounts[0],
      });
    } catch (exception) {
      hasRaisedException = true;
      assert.equal(
        exception.message.includes("Partner must be registered"),
        true
      );
    }
    assert.equal(hasRaisedException, true);
  });

  it("should not unauthorize an unregistered user", async () => {
    const ucdp = await Ucdp.deployed();

    let hasRaisedException = false;
    try {
      const partnerAddress = web3.utils.padLeft(0x123, 40);
      await ucdp.unauthorizePartner(partnerAddress, { from: accounts[1] });
    } catch (exception) {
      hasRaisedException = true;
      assert.equal(
        exception.message.includes("Sender must be registered as a User"),
        true
      );
    }
    assert.equal(hasRaisedException, true);
  });

  it("should not unauthorize an unregistered partner", async () => {
    const ucdp = await Ucdp.deployed();

    let hasRaisedException = false;
    try {
      const unregisteredPartnerAddress = web3.utils.padLeft(0x321, 40);
      await ucdp.unauthorizePartner(unregisteredPartnerAddress, {
        from: accounts[0],
      });
    } catch (exception) {
      hasRaisedException = true;
      assert.equal(
        exception.message.includes("Partner must be registered"),
        true
      );
    }
    assert.equal(hasRaisedException, true);
  });
});
