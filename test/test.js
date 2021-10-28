var chai = require("chai");
var chaiHttp = require("chai-http");
var expect = chai.expect;

chai.use(chaiHttp);

describe("Ucdp", () => {
  it("should post a test event", () => {
    chai
      .request("http://0.0.0.0:8080")
      .post("/v1/events")
      .send({
        partner: {
          id: "0x0000000000000000000000000000000000000123",
        },
        user: {
          id: "0x0000000000000000000000000000000000000456",
        },
        events: [
          {
            name: "test",
          },
        ],
      })
      .end(function (err, res) {
        expect(err).to.be.null;
        expect(res).to.have.status(200);
        expect(res).to.be.json;
        expect(res.body["token"]).to.be.not.undefined;
      });
  });
});
