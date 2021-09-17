import { LCDClient, MnemonicKey } from "@terra-money/terra.js";
import chalk from "chalk";
import { migrateContract } from "./helpers";

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
const contractAddress = "terra1yhnn0zxydfs8rls2eydfrqmaweh2emt4gsmx3u";

async function deployContract() {
  const cw20CodeId = 529;

  process.stdout.write("Migrating Rock Paper Scissors contract... ");

  const migrateResult = await migrateContract(
    terra,
    deployer,
    deployer,
    contractAddress,
    cw20CodeId,
    {}
  );

  console.log(chalk.green("Migrated!"));
  console.log(migrateResult);
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
