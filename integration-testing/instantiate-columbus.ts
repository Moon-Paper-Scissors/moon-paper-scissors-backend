import { LCDClient, MnemonicKey } from "@terra-money/terra.js";
import chalk from "chalk";
import { instantiateContract } from "./helpers";

// const encoder = new util.TextEncoder()
const pk = new MnemonicKey({
  mnemonic: process.env.mnemonic,
});

// connect to bombay testnet
const terra = new LCDClient({
  URL: "https://lcd.terra.dev",
  chainID: "columbus-5",
});

const deployer = terra.wallet(pk);
let contractAddress: string;

async function deployContract() {
  // 531
  const cw20CodeId = 531;

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

  console.log(chalk.yellow("\nStep 2. Setup"));

  await deployContract();
})();
