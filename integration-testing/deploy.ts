import { LocalTerra } from "@terra-money/terra.js";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import chalk from "chalk";
import * as path from "path";
import { instantiateContract, storeCode } from "./helpers";

chai.use(chaiAsPromised);
const { expect } = chai;

//----------------------------------------------------------------------------------------
// Variables
//----------------------------------------------------------------------------------------

const terra = new LocalTerra();
const deployer = terra.wallets.test1;
const user1 = terra.wallets.test2;
const user2 = terra.wallets.test3;

let contractAddress: string;

//----------------------------------------------------------------------------------------
// Setup
//----------------------------------------------------------------------------------------

async function deployContract() {
  // Step 1. Upload TerraSwap Token code
  process.stdout.write("Uploading RPS code... ");

  const cw20CodeId = await storeCode(
    terra,
    deployer,
    path.resolve(__dirname, "../artifacts/rockpaperscissors.wasm")
  );

  console.log(chalk.green("Done!"), `${chalk.blue("codeId")}=${cw20CodeId}`);

  // Step 2. Instantiate RockPaperScissors contract
  process.stdout.write("Instantiating Rock Paper Scissors contract... ");

  const instantiateResult = await instantiateContract(
    terra,
    deployer,
    deployer,
    cw20CodeId,
    {}
  );

  contractAddress = instantiateResult.logs[0].events[0].attributes[0].value;

  console.log(
    chalk.green("Deployed!"),
    `${chalk.blue("contractAddress")}=${contractAddress}`
  );
}

//----------------------------------------------------------------------------------------
// Main
//----------------------------------------------------------------------------------------

(async () => {
  console.log(chalk.yellow("\nStep 1. Info"));

  console.log(`Use ${chalk.cyan(deployer.key.accAddress)} as deployer`);
  console.log(`Use ${chalk.cyan(user1.key.accAddress)} as user 1`);
  console.log(`Use ${chalk.cyan(user2.key.accAddress)} as user 1`);

  console.log(chalk.yellow("\nStep 2. Setup"));

  await deployContract();
})();
