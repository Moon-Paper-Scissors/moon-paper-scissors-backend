import {
  LCDClient,
  MnemonicKey,
  MsgExecuteContract,
} from "@terra-money/terra.js";
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
import { QueryMsg } from "./types/query_msg";

chai.use(chaiAsPromised);
const { expect } = chai;

//----------------------------------------------------------------------------------------
// Variables
//----------------------------------------------------------------------------------------

// connect to bombay testnet
const terra = new LCDClient({
  URL: "https://bombay-lcd.terra.dev",
  chainID: "bombay-12",
});

const pk1 = new MnemonicKey({
  mnemonic: process.env.mnemonic,
});

const pk2 = new MnemonicKey({
  mnemonic: process.env.mnemonic2,
});

const deployer = terra.wallet(pk1);
const user1 = deployer;
const user2 = terra.wallet(pk2);

let contractAddress: string;
let user1InitFunds: string;
let user2InitFunds: string;

//

const nonce = "1";
const rock_move_hash = sha256("Rock" + nonce).toString();
const paper_move_hash = sha256("Paper" + nonce).toString();
const scissors_move_hash = sha256("Scissors" + nonce).toString();

//----------------------------------------------------------------------------------------
// Setup
//----------------------------------------------------------------------------------------

async function setupTest() {
  // Step 1. Upload TerraSwap Token code
  process.stdout.write("Uploading RPS code... ");

  const cw20CodeId = await storeCode(
    terra,
    deployer,
    path.resolve(__dirname, "../artifacts/rockpaperscissors.wasm")
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
// Test 1. Join a Game
//----------------------------------------------------------------------------------------

async function testJoinGame() {
  process.stdout.write("Should join game... ");

  console.log(await terra.bank.balance(user1.key.accAddress));

  const sendUser1Transaction = async (message: ExecuteMsg) => {
    await sendTransaction(terra, user1, [
      new MsgExecuteContract(user1.key.accAddress, contractAddress, message),
    ]);
  };

  const sendUser2Transaction = async (message: ExecuteMsg) => {
    await sendTransaction(terra, user2, [
      new MsgExecuteContract(user2.key.accAddress, contractAddress, message),
    ]);
  };

  const betAmount = "5000000";

  // START THE GAME
  const joinGameMessage: ExecuteMsg = {
    join_game: { num_hands_to_win: 1 },
  };
  await sendTransaction(terra, user1, [
    new MsgExecuteContract(
      user1.key.accAddress,
      contractAddress,
      joinGameMessage,
      { uluna: betAmount }
    ),
  ]);

  // JOIN THE GAME
  await sendTransaction(terra, user2, [
    new MsgExecuteContract(
      user2.key.accAddress,
      contractAddress,
      joinGameMessage,
      { uluna: betAmount }
    ),
  ]);
  const query_msg: QueryMsg = {
    get_game_by_player: {
      player: user1.key.accAddress,
    },
  };
  const res = (await terra.wasm.contractQuery(
    contractAddress,
    query_msg
  )) as GetGameByPlayerResponse;

  for (let i = 0; i < 3; i++) {
    // PLAYER 1 PLAYS A MOVE
    const player1_commit_message1: ExecuteMsg = {
      commit_move: {
        player1: user1.key.accAddress,
        player2: user2.key.accAddress,
        hashed_move: rock_move_hash,
      },
    };
    await sendUser1Transaction(player1_commit_message1);

    // PLAYER 2 PLAYS A MOVE
    const player2_commit_message1: ExecuteMsg = {
      commit_move: {
        player1: user1.key.accAddress,
        player2: user2.key.accAddress,
        hashed_move: paper_move_hash,
      },
    };
    await sendUser2Transaction(player2_commit_message1);

    // PLAYER 1 REVEALS MOVE
    const player1_reveal_message1: ExecuteMsg = {
      reveal_move: {
        player1: user1.key.accAddress,
        player2: user2.key.accAddress,
        game_move: "Rock",
        nonce: nonce,
      },
    };
    await sendUser1Transaction(player1_reveal_message1);

    // QUERY GAME STATUS
    const query_msg: QueryMsg = {
      get_game: {
        player1: user1.key.accAddress,
        player2: user2.key.accAddress,
      },
    };

    // PLAYER 2 REVEALS MOVE
    const player2_reveal_message1: ExecuteMsg = {
      reveal_move: {
        player1: user1.key.accAddress,
        player2: user2.key.accAddress,
        game_move: "Paper",
        nonce: nonce,
      },
    };
    await sendUser2Transaction(player2_reveal_message1);

    // QUERY GAME STATUS
    const res = await terra.wasm.contractQuery(contractAddress, query_msg);
    console.log(res);
  }

  // confirm that player 1 lost the bet amount
  expect(
    await queryNativeTokenBalance(terra, user1.key.accAddress, "uluna")
  ).to.equal((+user1InitFunds - +betAmount).toString());

  // confirm that player2 has made the bet amount
  expect(
    await queryNativeTokenBalance(terra, user2.key.accAddress, "uluna")
  ).to.equal((+user2InitFunds + +betAmount).toString());

  // confirm that the smart contract doesn't have any remaining funds
  expect(
    await queryNativeTokenBalance(terra, contractAddress, "uluna")
  ).to.equal("0");

  console.log(chalk.green("Passed!"));
}

// async function testPlayGame() {
//   process.stdout.write("Should play game... ");

//   console.log(await terra.bank.balance(user1.key.accAddress));

//   const sendUser1Transaction = async (message: ExecuteMsg) => {
//     await sendTransaction(terra, user1, [
//       new MsgExecuteContract(user1.key.accAddress, contractAddress, message),
//     ]);
//   };

//   const sendUser2Transaction = async (message: ExecuteMsg) => {
//     await sendTransaction(terra, user2, [
//       new MsgExecuteContract(user2.key.accAddress, contractAddress, message),
//     ]);
//   };

//   const betAmount = "5000000";

//   // START THE GAME
//   const startGameMessage: ExecuteMsg = {
//     start_game: {
//       player2: user2.key.accAddress,
//     },
//   };
//   await sendTransaction(terra, user1, [
//     new MsgExecuteContract(
//       user1.key.accAddress,
//       contractAddress,
//       startGameMessage,
//       { uluna: betAmount }
//     ),
//   ]);

//   // JOIN THE GAME
//   const joinGameMessage: ExecuteMsg = {
//     join_game: {
//       player1: user1.key.accAddress,
//     },
//   };
//   await sendTransaction(terra, user2, [
//     new MsgExecuteContract(
//       user2.key.accAddress,
//       contractAddress,
//       joinGameMessage,
//       { uluna: betAmount }
//     ),
//   ]);

//   const nonce = "1";
//   const rock_move_hash = sha256("Rock" + nonce).toString();
//   const paper_move_hash = sha256("Paper" + nonce).toString();
//   const scissors_move_hash = sha256("Scissors" + nonce).toString();

//   for (let i = 0; i < 3; i++) {
//     // PLAYER 1 PLAYS A MOVE
//     const player1_commit_message1: ExecuteMsg = {
//       commit_move: {
//         player1: user1.key.accAddress,
//         player2: user2.key.accAddress,
//         hashed_move: rock_move_hash,
//       },
//     };
//     await sendUser1Transaction(player1_commit_message1);

//     // PLAYER 2 PLAYS A MOVE
//     const player2_commit_message1: ExecuteMsg = {
//       commit_move: {
//         player1: user1.key.accAddress,
//         player2: user2.key.accAddress,
//         hashed_move: paper_move_hash,
//       },
//     };
//     await sendUser2Transaction(player2_commit_message1);

//     // PLAYER 1 REVEALS MOVE
//     const player1_reveal_message1: ExecuteMsg = {
//       reveal_move: {
//         player1: user1.key.accAddress,
//         player2: user2.key.accAddress,
//         game_move: "Rock",
//         nonce: nonce,
//       },
//     };
//     await sendUser1Transaction(player1_reveal_message1);

//     // QUERY GAME STATUS
//     const query_msg: QueryMsg = {
//       get_game: {
//         player1: user1.key.accAddress,
//         player2: user2.key.accAddress,
//       },
//     };

//     // PLAYER 2 REVEALS MOVE
//     const player2_reveal_message1: ExecuteMsg = {
//       reveal_move: {
//         player1: user1.key.accAddress,
//         player2: user2.key.accAddress,
//         game_move: "Paper",
//         nonce: nonce,
//       },
//     };
//     await sendUser2Transaction(player2_reveal_message1);

//     // QUERY GAME STATUS
//     const res = await terra.wasm.contractQuery(contractAddress, query_msg);
//     console.log(res);
//   }

//   // confirm that player 1 lost the bet amount
//   expect(
//     await queryNativeTokenBalance(terra, user1.key.accAddress, "uluna")
//   ).to.equal((+user1InitFunds - +betAmount).toString());

//   // confirm that player2 has made the bet amount
//   expect(
//     await queryNativeTokenBalance(terra, user2.key.accAddress, "uluna")
//   ).to.equal((+user2InitFunds + +betAmount).toString());

//   // confirm that the smart contract doesn't have any remaining funds
//   expect(
//     await queryNativeTokenBalance(terra, contractAddress, "uluna")
//   ).to.equal("0");

//   console.log(chalk.green("Passed!"));
// }

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
  console.log(`Use ${chalk.cyan(user2.key.accAddress)} as user 1`);

  console.log(chalk.yellow("\nStep 2. Setup"));

  await setupTest();

  //   console.log(chalk.yellow("\nStep 3. Tests"));

  await testJoinGame();
  //   await testPlayGame();
  //   await testSwap();
  //   await testSlippage();

  //   console.log("");
})();
