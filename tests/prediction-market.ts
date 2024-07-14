import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { assert } from "chai";
import { PredictionMarket } from "../target/types/prediction_market";

const BET_SEED = "bet";
const HIGHER_POOL_SEED = "higher_pool";
const LOWER_POOL_SEED = "lower_pool";
const USDC_MINT = "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"; // Example USDC Mint address

describe("prediction_market", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace
    .PredictionMarket as Program<PredictionMarket>;

  describe("Initialize Market", () => {
    const targetPrice = new anchor.BN(140);
    const feedIdString =
      "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";
    const feedId = Array.from(Buffer.from(feedIdString, "utf-8"));
    const marketDuration = new anchor.BN(1300);

    it("Initializes a market", async () => {
      const marketCreator = provider.wallet.publicKey;
      await airdrop(provider.connection, marketCreator);

      const [marketAddress, marketBump] = getMarketAddress(
        marketCreator,
        targetPrice,
        marketDuration,
        program.programId
      );

      await program.methods
        .initializeMarket(targetPrice, Array.from(feedId), marketDuration)
        .accounts({
          marketCreator: marketCreator,
        })
        .rpc();

      await checkMarket(
        program,
        marketAddress,
        marketCreator,
        Array.from(feedId),
        targetPrice,
        marketDuration,
        marketBump
      );
    });
  });
});

async function airdrop(
  connection: any,
  address: any,
  amount = 10 * LAMPORTS_PER_SOL
) {
  await connection.confirmTransaction(
    await connection.requestAirdrop(address, amount),
    "confirmed"
  );
}

function getMarketAddress(
  creator: PublicKey,
  targetPrice: anchor.BN,
  marketDuration: anchor.BN,
  programID: PublicKey
) {
  return PublicKey.findProgramAddressSync(
    [
      creator.toBuffer(),
      targetPrice.toArrayLike(Buffer, "le", 8),
      marketDuration.toArrayLike(Buffer, "le", 8),
    ],
    programID
  );
}

async function checkMarket(
  program: anchor.Program<PredictionMarket>,
  marketAddress: anchor.web3.PublicKey,
  marketCreator: anchor.web3.PublicKey,
  feedId: number[],
  targetPrice: anchor.BN,
  marketDuration: anchor.BN,
  bump: number
) {
  const marketData = await program.account.market.fetch(marketAddress);

  assert.ok(marketData.creator.equals(marketCreator));
  assert.ok(marketData.targetPrice.eq(targetPrice));
  assert.ok(marketData.marketDuration.eq(marketDuration));
  assert.deepEqual(marketData.initialization, { initializedMarket: {} });
  assert.deepEqual(marketData.bump, bump);
}
