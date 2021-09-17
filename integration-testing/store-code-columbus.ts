import { LCDClient, MnemonicKey } from "@terra-money/terra.js";
import chalk from "chalk";
import * as path from "path";
import { storeCode } from "./helpers";

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
  // Step 1. Upload RPS Code
  process.stdout.write("Uploading RPS code... ");

  const cw20CodeId = await storeCode(
    terra,
    deployer,
    path.resolve(__dirname, "../artifacts/cw_rockpaperscissors.wasm")
  );

  console.log(chalk.green("Done!"), `${chalk.blue("codeId")}=${cw20CodeId}`);
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
