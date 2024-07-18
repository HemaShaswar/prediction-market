import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { assert } from "chai";
import { PredictionMarket } from "../target/types/prediction_market";
import crypto, { generateKey, getCipherInfo, Sign } from "crypto";
import * as token from "@solana/spl-token";
import { PythSolanaReceiver } from "@pythnetwork/pyth-solana-receiver";

const BET_SEED = "bet";
const HIGHER_POOL_SEED = "higher_pool";
const LOWER_POOL_SEED = "lower_pool";
const USDC_MINT = "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"; // Example USDC Mint address

describe("prediction_market", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace
    .PredictionMarket as Program<PredictionMarket>;

  const feedIdString: string =
    "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";
  const feedIdString2: string = "Invalid FeedId Length";

  const targetPrice: anchor.BN = new anchor.BN(140);
  const marketDuration: anchor.BN = new anchor.BN(1300);
  const marketCreator1 = anchor.web3.Keypair.generate();

  const hema = anchor.web3.Keypair.generate();
  const mint_authority = anchor.web3.Keypair.generate();

  const to_mint = new anchor.BN(10000000);

  describe("Market Initialization", () => {
    it("Initializes a market", async () => {
      await airdrop(provider.connection, marketCreator1.publicKey);

      const [marketAddress, marketBump] = getMarketAddress(
        marketCreator1.publicKey,
        feedIdString,
        targetPrice,
        marketDuration,
        program.programId
      );

      await program.methods
        .initializeMarket(targetPrice, feedIdString, marketDuration)
        .accountsStrict({
          market: marketAddress,
          marketCreator: marketCreator1.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([marketCreator1])
        .rpc({ commitment: "confirmed" });

      await checkMarket(
        program,
        marketAddress,
        marketCreator1.publicKey,
        feedIdString,
        targetPrice,
        marketDuration,
        marketBump
      );
    });
    it("Can not initialize with invalid FeedId", async () => {
      const marketCreator = provider.wallet.publicKey;

      await airdrop(provider.connection, marketCreator);

      let should_fail = "This Should Fail";
      try {
        const [marketAddress, marketBump] = getMarketAddress(
          marketCreator,
          feedIdString2,
          targetPrice,
          marketDuration,
          program.programId
        );

        await program.methods
          .initializeMarket(targetPrice, feedIdString2, marketDuration)
          .accountsStrict({
            marketCreator: marketCreator,
            market: marketAddress,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .rpc({ commitment: "confirmed" });

        await checkMarket(
          program,
          marketAddress,
          marketCreator,
          feedIdString,
          targetPrice,
          marketDuration,
          marketBump
        );
      } catch (e) {
        assert.strictEqual(e.error.errorCode.code, "IncorrectFeedIDLength");
        should_fail = "Failed";
      }
      assert.strictEqual(should_fail, "Failed");
    });
  });
  describe("Pool Initialization", () => {
    it("Initialize pool mint and token accounts", async () => {
      await airdrop(provider.connection, marketCreator1.publicKey);
      await airdrop(provider.connection, mint_authority.publicKey);

      const mint = await token.createMint(
        provider.connection,
        mint_authority,
        mint_authority.publicKey,
        null,
        6
      );

      const user_ata = await token.getOrCreateAssociatedTokenAccount(
        provider.connection,
        marketCreator1,
        mint,
        marketCreator1.publicKey
      );

      await token.mintTo(
        provider.connection,
        mint_authority,
        mint,
        user_ata.address,
        mint_authority,
        to_mint.toNumber()
      );

      const [marketAddress, marketBump] = getMarketAddress(
        marketCreator1.publicKey,
        feedIdString,
        targetPrice,
        marketDuration,
        program.programId
      );

      const [higherPoolAddress, higherPoolBump] = getPoolAddress(
        HIGHER_POOL_SEED,
        marketAddress,
        program.programId
      );
      const [lowerPoolAddress, LowerpoolBump] = getPoolAddress(
        LOWER_POOL_SEED,
        marketAddress,
        program.programId
      );

      console.log("mint account in initialization: ", mint.toString());

      await program.methods
        .initializePools()
        .accountsStrict({
          market: marketAddress,
          marketCreator: marketCreator1.publicKey,
          poolTokenMint: mint,
          higherPool: higherPoolAddress,
          lowerPool: lowerPoolAddress,
          userAta: user_ata.address,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: token.TOKEN_PROGRAM_ID,
        })
        .signers([marketCreator1])
        .rpc({ commitment: "confirmed" });

      await checkMarket(
        program,
        marketAddress,
        marketCreator1.publicKey,
        feedIdString,
        targetPrice,
        marketDuration,
        marketBump,
        higherPoolBump,
        LowerpoolBump,
        mint
      );
    });
  });
  describe("Cancel Market", () => {
    it("Market Canceled", async () => {
      await airdrop(provider.connection, marketCreator1.publicKey);

      const [marketAddress, marketBump] = getMarketAddress(
        marketCreator1.publicKey,
        feedIdString,
        targetPrice,
        marketDuration,
        program.programId
      );

      const market = await program.account.market.fetch(marketAddress);

      console.log("mint account in cancelation: ", market.mint.toString());
      const creator_ata = await token.getOrCreateAssociatedTokenAccount(
        provider.connection,
        marketCreator1,
        market.mint,
        marketCreator1.publicKey
      );

      const [higherPoolAddress, higherPoolBump] = getPoolAddress(
        HIGHER_POOL_SEED,
        marketAddress,
        program.programId
      );
      const [lowerPoolAddress, LowerpoolBump] = getPoolAddress(
        LOWER_POOL_SEED,
        marketAddress,
        program.programId
      );

      await program.methods
        .cancelMarket()
        .accountsStrict({
          market: marketAddress,
          marketCreator: marketCreator1.publicKey,
          higherPool: higherPoolAddress,
          lowerPool: lowerPoolAddress,
          creatorAta: creator_ata.address,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: token.TOKEN_PROGRAM_ID,
        })
        .signers([marketCreator1])
        .rpc({ commitment: "confirmed" });
    });
  });
});

async function airdrop(
  connection: anchor.web3.Connection,
  address: PublicKey,
  amount = 3 * LAMPORTS_PER_SOL
) {
  await connection.confirmTransaction(
    await connection.requestAirdrop(address, amount),
    "confirmed"
  );
}

function getMarketAddress(
  creator: PublicKey,
  feedId: string,
  targetPrice: anchor.BN,
  marketDuration: anchor.BN,
  programID: PublicKey
) {
  let hexString = crypto
    .createHash("sha256")
    .update(feedId, "utf-8")
    .digest("hex");
  let feed_seed = Uint8Array.from(Buffer.from(hexString, "hex"));

  return PublicKey.findProgramAddressSync(
    [
      creator.toBuffer(),
      feed_seed,
      targetPrice.toArrayLike(Buffer, "le", 8),
      marketDuration.toArrayLike(Buffer, "le", 8),
    ],
    programID
  );
}

function getPoolAddress(
  poolStringSeed: string,
  marketAddress: PublicKey,
  programId: PublicKey
) {
  return PublicKey.findProgramAddressSync(
    [anchor.utils.bytes.utf8.encode(poolStringSeed), marketAddress.toBuffer()],
    programId
  );
}

async function checkMarket(
  program: anchor.Program<PredictionMarket>,
  marketAddress: PublicKey,
  marketCreator: PublicKey,
  feedId: string,
  targetPrice: anchor.BN,
  marketDuration: anchor.BN,
  bump: number,
  higherPoolBump?: number,
  lowerPoolBump?: number,
  mint?: PublicKey
) {
  const marketData = await program.account.market.fetch(marketAddress);

  assert.strictEqual(marketData.creator.toString(), marketCreator.toString());
  assert.strictEqual(marketData.targetPrice.toString(), targetPrice.toString());
  assert.strictEqual(
    marketData.marketDuration.toString(),
    marketDuration.toString()
  );
  assert.strictEqual(marketData.bump.toString(), bump.toString());

  assert.deepEqual(marketData.initialization, { initializedMarket: {} });

  const utf8ByteArray_content = stringToUtf8ByteArray(feedId);
  const paddedByteArray_content = padByteArrayWithZeroes(
    utf8ByteArray_content,
    66
  );
  assert.strictEqual(
    marketData.feedId.toString(),
    paddedByteArray_content.toString()
  );

  if (higherPoolBump) {
    assert.strictEqual(
      higherPoolBump.toString(),
      marketData.higherPoolBump.toString()
    );
  }
  if (lowerPoolBump) {
    assert.strictEqual(
      marketData.lowerPoolBump.toString(),
      lowerPoolBump.toString()
    );
  }

  if (mint) {
    assert.strictEqual(marketData.mint.toString(), mint.toString());
  }
}

function stringToUtf8ByteArray(inputString: string): Uint8Array {
  const encoder = new TextEncoder();
  return encoder.encode(inputString);
}

// Function to pad a byte array with zeroes to a specified length
function padByteArrayWithZeroes(
  byteArray: Uint8Array,
  length: number
): Uint8Array {
  if (byteArray.length >= length) {
    return byteArray;
  }
  const paddedArray = new Uint8Array(length);
  paddedArray.set(byteArray, 0);
  return paddedArray;
}
