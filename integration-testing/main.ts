import { LocalTerra, MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import chalk from "chalk";
import sha256 from "crypto-js/sha256";
import * as path from "path";
import {
  instantiateContract,
  queryNativeTokenBalance,
  sendTransaction,
  storeCode,
} from "./helpers";
import { ExecuteMsg } from "./types/execute_msg";
import { GetGameByPlayerResponse } from "./types/get_game_by_player_response";
import { GetLeaderboardResponse } from "./types/get_leaderboard_response";
import { QueryMsg } from "./types/query_msg";

chai.use(chaiAsPromised);
const { expect } = chai;

//----------------------------------------------------------------------------------------
// Variables
//----------------------------------------------------------------------------------------

const terra = new LocalTerra();
const deployer = terra.wallets.test1;
const user1 = terra.wallets.test2;
const user2 = terra.wallets.test3;
const user3 = terra.wallets.test4;
const user4 = terra.wallets.test5;

// game constants
const nonce = "1";
const rock_move_hash = sha256("Rock" + nonce).toString();
const paper_move_hash = sha256("Paper" + nonce).toString();
const scissors_move_hash = sha256("Scissors" + nonce).toString();

let contractAddress: string;
let user1InitFunds: string;
let user2InitFunds: string;
let query_msg: QueryMsg;

//----------------------------------------------------------------------------------------
// Setup
//----------------------------------------------------------------------------------------

async function setupTest() {
  // Step 1. Upload TerraSwap Token code
  process.stdout.write("Uploading TerraSwap Token code... ");

  const cw20CodeId = await storeCode(
    terra,
    deployer,
    path.resolve(__dirname, "../artifacts/cw_rockpaperscissors.wasm")
  );

  console.log(chalk.green("Done!"), `${chalk.blue("codeId")}=${cw20CodeId}`);

  // Step 2. Instantiate TerraSwap Token contract
  process.stdout.write("Instantiating Rock Paper Scissors contract... ");

  const instantiateResult = await instantiateContract(
    terra,
    deployer,
    deployer,
    cw20CodeId,
    {}
  );

  contractAddress = instantiateResult.logs[0].events[0].attributes[0].value;
  user1InitFunds = await queryNativeTokenBalance(
    terra,
    user1.key.accAddress,
    "uluna"
  );
  user2InitFunds = await queryNativeTokenBalance(
    terra,
    user2.key.accAddress,
    "uluna"
  );
}

//----------------------------------------------------------------------------------------
// Test 1. Play a game
//----------------------------------------------------------------------------------------

async function testPlayGame() {
  process.stdout.write("Should play game... ");

  console.log(await terra.bank.balance(user1.key.accAddress));
  console.log(await terra.bank.balance(user2.key.accAddress));
  console.log(await terra.bank.balance(user3.key.accAddress));
  console.log(await terra.bank.balance(user4.key.accAddress));

  const betAmount = "5000000";

  // START THE GAME

  // play a couple hands
  const playHand = async (player1: Wallet, player2: Wallet) => {
    console.log(
      "Playing hand between ",
      player1.key.accAddress,
      player2.key.accAddress
    );

    const joinGameMessage: ExecuteMsg = {
      join_game: { num_hands_to_win: 1 },
    };
    await sendTransaction(terra, player1, [
      new MsgExecuteContract(
        player1.key.accAddress,
        contractAddress,
        joinGameMessage,
        { uluna: betAmount }
      ),
    ]);

    await sendTransaction(terra, player2, [
      new MsgExecuteContract(
        player2.key.accAddress,
        contractAddress,
        joinGameMessage,
        { uluna: betAmount }
      ),
    ]);

    query_msg = {
      get_game_by_player: {
        player: player1.key.accAddress,
      },
    };

    const gameByPlayerRes = (await terra.wasm.contractQuery(
      contractAddress,
      query_msg
    )) as GetGameByPlayerResponse;
    const player1_commit_message1: ExecuteMsg = {
      commit_move: {
        player1: player1.key.accAddress,
        player2: player2.key.accAddress,
        hashed_move: rock_move_hash,
      },
    };

    const player2_commit_message1: ExecuteMsg = {
      commit_move: {
        player1: player1.key.accAddress,
        player2: player2.key.accAddress,
        hashed_move: paper_move_hash,
      },
    };

    // player 1 commit
    await sendTransaction(terra, player1, [
      new MsgExecuteContract(
        player1.key.accAddress,
        contractAddress,
        player1_commit_message1
      ),
    ]);

    // player 2 commit
    await sendTransaction(terra, player2, [
      new MsgExecuteContract(
        player2.key.accAddress,
        contractAddress,
        player2_commit_message1
      ),
    ]);

    console.log("Committed successfully");

    // player 1 reveal
    const player1_reveal_message1: ExecuteMsg = {
      reveal_move: {
        player1: player1.key.accAddress,
        player2: player2.key.accAddress,
        game_move: "Rock",
        nonce: nonce,
      },
    };

    // player 2 reveal
    const player2_reveal_message1: ExecuteMsg = {
      reveal_move: {
        player1: player1.key.accAddress,
        player2: player2.key.accAddress,
        game_move: "Paper",
        nonce: nonce,
      },
    };

    // player 1 reveal
    await sendTransaction(terra, player1, [
      new MsgExecuteContract(
        player1.key.accAddress,
        contractAddress,
        player1_reveal_message1
      ),
    ]);

    // player 2 reveal
    await sendTransaction(terra, player2, [
      new MsgExecuteContract(
        player2.key.accAddress,
        contractAddress,
        player2_reveal_message1
      ),
    ]);
    console.log("Revealed successfully");
  };

  await playHand(user1, user2);
  await playHand(user3, user4);
  await playHand(user3, user4);
  await playHand(user4, user1);
  await playHand(user2, user3);

  // get the leaderboard
  query_msg = {
    get_leaderboard: {},
  };

  const leaderboardRes = (await terra.wasm.contractQuery(
    contractAddress,
    query_msg
  )) as GetLeaderboardResponse;

  console.log(leaderboardRes);

  console.log(chalk.green("Passed!"));
}

//----------------------------------------------------------------------------------------
// Test 2. Swap
//
// User 2 sells 1 MIR for UST
//
// k = poolUMir * poolUUsd
// = 69000000 * 420000000 = 28980000000000000
// returnAmount = poolUusd - k / (poolUMir + offerUMir)
// = 420000000 - 28980000000000000 / (69000000 + 1000000)
// = 6000000
// fee = returnAmount * feeRate
// = 6000000 * 0.003
// = 18000
// returnAmountAfterFee = returnUstAmount - fee
// = 6000000 - 18000
// = 5982000
// returnAmountAfterFeeAndTax = deductTax(5982000) = 5976023
// transaction cost for pool = addTax(5976023) = 5981999
//
// Result
// ---
// pool uMIR  69000000 + 1000000 = 70000000
// pool uusd  420000000 - 5981999 = 414018001
// user uLP   170235131
// user uMIR  10000000000 - 1000000 = 9999000000
// user uusd  balanceBeforeSwap + 5976023 - 4500000 (gas)
//----------------------------------------------------------------------------------------

// async function testSwap() {
//   process.stdout.write("Should swap... ");

//   const userUusdBefore = await queryNativeTokenBalance(
//     terra,
//     user2.key.accAddress,
//     "uusd"
//   );

//   await sendTransaction(terra, user2, [
//     new MsgExecuteContract(user2.key.accAddress, mirrorToken, {
//       send: {
//         amount: "1000000",
//         contract: terraswapPair,
//         msg: toEncodedBinary({
//           swap: {},
//         }),
//       },
//     }),
//   ]);

//   const poolUMir = await queryTokenBalance(terra, terraswapPair, mirrorToken);
//   expect(poolUMir).to.equal("70000000");

//   const poolUUsd = await queryNativeTokenBalance(terra, terraswapPair, "uusd");
//   expect(poolUUsd).to.equal("414018001");

//   const userULp = await queryTokenBalance(
//     terra,
//     user1.key.accAddress,
//     terraswapLpToken
//   );
//   expect(userULp).to.equal("170235131");

//   const userUMir = await queryTokenBalance(
//     terra,
//     user2.key.accAddress,
//     mirrorToken
//   );
//   expect(userUMir).to.equal("9999000000");

//   const userUusdExpected = new BN(userUusdBefore)
//     .add(new BN("5976023"))
//     .sub(new BN("4500000"))
//     .toString();

//   const userUUsd = await queryNativeTokenBalance(
//     terra,
//     user2.key.accAddress,
//     "uusd"
//   );
//   expect(userUUsd).to.equal(userUusdExpected);

//   console.log(chalk.green("Passed!"));
// }

// //----------------------------------------------------------------------------------------
// // Test 3. Slippage tolerance
// //
// // User 2 tries to swap a large amount of MIR (say 50 MIR, while the pool only has 70) to
// // UST with a low max spread. The transaction should fail
// //----------------------------------------------------------------------------------------

// async function testSlippage() {
//   process.stdout.write("Should check max spread... ");

//   await expect(
//     sendTransaction(terra, user2, [
//       new MsgExecuteContract(user2.key.accAddress, mirrorToken, {
//         send: {
//           amount: "50000000",
//           contract: terraswapPair,
//           msg: toEncodedBinary({
//             swap: {
//               max_spread: "0.01",
//             },
//           }),
//         },
//       }),
//     ])
//   ).to.be.rejectedWith("Max spread assertion");

//   console.log(chalk.green("Passed!"));
// }

//----------------------------------------------------------------------------------------
// Main
//----------------------------------------------------------------------------------------

(async () => {
  console.log(chalk.yellow("\nStep 1. Info"));

  console.log(`Use ${chalk.cyan(deployer.key.accAddress)} as deployer`);
  console.log(`Use ${chalk.cyan(user1.key.accAddress)} as user 1`);
  console.log(`Use ${chalk.cyan(user2.key.accAddress)} as user 2`);
  console.log(`Use ${chalk.cyan(user3.key.accAddress)} as user 3`);
  console.log(`Use ${chalk.cyan(user4.key.accAddress)} as user 4`);

  console.log(chalk.yellow("\nStep 2. Setup"));

  await setupTest();

  //   console.log(chalk.yellow("\nStep 3. Tests"));

  await testPlayGame();
  //   await testSwap();
  //   await testSlippage();

  //   console.log("");
})();
