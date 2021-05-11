import { expect } from "chai";
import { describeDevThales } from "../util/setup-dev-tests";

import { GENESIS_ACCOUNT } from "../util/constants";
import { getCompiled } from "../util/contracts";

describeDevThales("Estimate Gas - Contract creation", (context) => {
  it("should return contract creation gas cost", async function () {
    const contract = await getCompiled("TestContract");
    expect(
      await context.web3.eth.estimateGas({
        from: GENESIS_ACCOUNT,
        data: contract.byteCode,
      })
    ).to.equal(149143);
  });
});
