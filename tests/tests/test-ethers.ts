import { expect } from "chai";
import { ethers } from "ethers";
import { GENESIS_ACCOUNT_PRIVATE_KEY } from "../util/constants";
import { describeDevThales } from "../util/setup-dev-tests";
import { getCompiled } from "../util/contracts";

describeDevThales("Ethers.js", (context) => {
  it("should get correct network ids", async function () {
    expect((await context.ethers.getNetwork()).chainId).to.equal(1281);
  });
});

describeDevThales("Ethers.js contract", (context) => {
  it("should be deployable", async function () {
    let signer = new ethers.Wallet(GENESIS_ACCOUNT_PRIVATE_KEY, context.ethers);
    const contractData = await getCompiled("TestContract");
    const contractFactory = new ethers.ContractFactory(
      contractData.contract.abi as ethers.ContractInterface,
      contractData.byteCode,
      signer
    );

    // Must create the block and then wait, because etherjs will wait until
    // the contract is mined to return;
    let contract = await new Promise<ethers.Contract>(async (resolve) => {
      const contractPromise = contractFactory.deploy();
      await context.createBlock();
      resolve(await contractPromise);
    });

    expect(contract.address);
    expect(await context.web3.eth.getCode(contract.address)).to.be.string;
  });
});

describeDevThales("Ethers.js contract", (context) => {
  it("should be callable", async function () {
    let signer = new ethers.Wallet(GENESIS_ACCOUNT_PRIVATE_KEY, context.ethers);
    const contractData = await getCompiled("TestContract");
    const contractFactory = new ethers.ContractFactory(
      contractData.contract.abi as ethers.ContractInterface,
      contractData.byteCode,
      signer
    );
    let contract = await new Promise<ethers.Contract>(async (resolve) => {
      const contractPromise = contractFactory.deploy();
      await context.createBlock();
      resolve(await contractPromise);
    });

    // Must create the block and then wait, because etherjs will wait until
    // the contract is mined to return;
    let result = await new Promise<string>(async (resolve) => {
      const callPromise = contract.multiply(3);
      await context.createBlock();
      resolve(await callPromise);
    });
    expect(result.toString()).to.equal("21");

    // Instantiate contract from address
    const contractFromAddress = new ethers.Contract(
      contract.address,
      contractData.contract.abi as ethers.ContractInterface,
      signer
    );
    expect((await contractFromAddress.multiply(3)).toString()).to.equal("21");
  });
});
