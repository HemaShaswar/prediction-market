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

  const to_mint = new anchor.BN(30000000);

  const INITIAL_USDC_AMOUNT = program.idl.constants.find(
    (el) => el.name == "initialUsdcPoolAmount"
  ).value;

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
        .rpc()
        .then(confirmTx);

      await checkMarket(
        program,
        marketAddress,
        marketCreator1.publicKey,
        feedIdString,
        targetPrice,
        marketDuration,
        marketBump,
        { initializedMarket: {} }
      );
    });
    it("Can not initialize with invalid FeedId", async () => {
      await airdrop(provider.connection, marketCreator1.publicKey);

      let should_fail = "This Should Fail";
      try {
        const [marketAddress, marketBump] = getMarketAddress(
          marketCreator1.publicKey,
          feedIdString2,
          targetPrice,
          marketDuration,
          program.programId
        );

        await program.methods
          .initializeMarket(targetPrice, feedIdString2, marketDuration)
          .accountsStrict({
            marketCreator: marketCreator1.publicKey,
            market: marketAddress,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([marketCreator1])
          .rpc()
          .then(confirmTx);
      } catch (e) {
        const anchorErr = anchor.AnchorError.parse(e.logs);
        assert.strictEqual(
          anchorErr.error.errorCode.code,
          "IncorrectFeedIDLength",
          "Unexpected Error Code"
        );
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

      const userAta = await token.getOrCreateAssociatedTokenAccount(
        provider.connection,
        marketCreator1,
        mint,
        marketCreator1.publicKey
      );

      await token.mintTo(
        provider.connection,
        mint_authority,
        mint,
        userAta.address,
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

      const [lowerPoolAddress, lowerPoolBump] = getPoolAddress(
        LOWER_POOL_SEED,
        marketAddress,
        program.programId
      );

      await program.methods
        .initializePools()
        .accountsStrict({
          market: marketAddress,
          marketCreator: marketCreator1.publicKey,
          poolTokenMint: mint,
          higherPool: higherPoolAddress,
          lowerPool: lowerPoolAddress,
          userAta: userAta.address,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: token.TOKEN_PROGRAM_ID,
        })
        .signers([marketCreator1])
        .rpc()
        .then(confirmTx);

      await checkMarket(
        program,
        marketAddress,
        marketCreator1.publicKey,
        feedIdString,
        targetPrice,
        marketDuration,
        marketBump,
        { initializedPools: {} },
        higherPoolBump,
        lowerPoolBump,
        mint
      );
    });
  });

  describe("Place Bet", () => {
    const betAmount = new anchor.BN(10000);
    const betDirection = { higher: {} };
    it("Placed Bet", async () => {
      await airdrop(provider.connection, hema.publicKey);
      await airdrop(provider.connection, marketCreator1.publicKey);

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

      const [lowerPoolAddress, lowerPoolBump] = getPoolAddress(
        LOWER_POOL_SEED,
        marketAddress,
        program.programId
      );

      const market = await program.account.market.fetch(marketAddress);

      const userAta = await token.getOrCreateAssociatedTokenAccount(
        provider.connection,
        hema,
        market.mint,
        hema.publicKey
      );

      await token.mintTo(
        provider.connection,
        mint_authority,
        market.mint,
        userAta.address,
        mint_authority,
        to_mint.toNumber()
      );

      const [betAddress, betBump] = getBetAddress(
        marketAddress,
        hema.publicKey,
        betAmount,
        betDirection,
        program.programId
      );

      await program.methods
        .placeBet(betAmount, betDirection)
        .accountsStrict({
          bet: betAddress,
          market: marketAddress,
          user: hema.publicKey,
          higherPool: higherPoolAddress,
          lowerPool: lowerPoolAddress,
          userAta: userAta.address,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: token.TOKEN_PROGRAM_ID,
        })
        .signers([hema])
        .rpc()
        .then(confirmTx);
    });
  });

  describe("Cancel Market", () => {
    it("Initializes market again", async () => {
      await airdrop(provider.connection, hema.publicKey);

      const [marketAddress, marketBump] = getMarketAddress(
        hema.publicKey,
        feedIdString,
        targetPrice,
        marketDuration,
        program.programId
      );

      await program.methods
        .initializeMarket(targetPrice, feedIdString, marketDuration)
        .accountsStrict({
          market: marketAddress,
          marketCreator: hema.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([hema])
        .rpc()
        .then(confirmTx);

      await checkMarket(
        program,
        marketAddress,
        hema.publicKey,
        feedIdString,
        targetPrice,
        marketDuration,
        marketBump,
        { initializedMarket: {} }
      );
    });

    it("Initialize pool again", async () => {
      await airdrop(provider.connection, hema.publicKey);
      await airdrop(provider.connection, mint_authority.publicKey);

      const mint = await token.createMint(
        provider.connection,
        mint_authority,
        mint_authority.publicKey,
        null,
        9
      );

      const userAta = await token.getOrCreateAssociatedTokenAccount(
        provider.connection,
        hema,
        mint,
        hema.publicKey
      );

      await token.mintTo(
        provider.connection,
        mint_authority,
        mint,
        userAta.address,
        mint_authority,
        to_mint.toNumber()
      );

      const [marketAddress, marketBump] = getMarketAddress(
        hema.publicKey,
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

      const [lowerPoolAddress, lowerPoolBump] = getPoolAddress(
        LOWER_POOL_SEED,
        marketAddress,
        program.programId
      );

      await program.methods
        .initializePools()
        .accountsStrict({
          market: marketAddress,
          marketCreator: hema.publicKey,
          poolTokenMint: mint,
          higherPool: higherPoolAddress,
          lowerPool: lowerPoolAddress,
          userAta: userAta.address,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: token.TOKEN_PROGRAM_ID,
        })
        .signers([hema])
        .rpc()
        .then(confirmTx);

      await checkMarket(
        program,
        marketAddress,
        hema.publicKey,
        feedIdString,
        targetPrice,
        marketDuration,
        marketBump,
        { initializedPools: {} },
        higherPoolBump,
        lowerPoolBump,
        mint
      );
    });
    it("Market Canceled", async () => {
      await airdrop(provider.connection, hema.publicKey);

      const [marketAddress, marketBump] = getMarketAddress(
        hema.publicKey,
        feedIdString,
        targetPrice,
        marketDuration,
        program.programId
      );
      const market = await program.account.market.fetch(marketAddress);

      const [higherPoolAddress, higherPoolBump] = getPoolAddress(
        HIGHER_POOL_SEED,
        marketAddress,
        program.programId
      );
      const [lowerPoolAddress, lowerPoolBump] = getPoolAddress(
        LOWER_POOL_SEED,
        marketAddress,
        program.programId
      );
      const creatorAta = await token.getOrCreateAssociatedTokenAccount(
        provider.connection,
        hema,
        market.mint,
        hema.publicKey
      );

      await program.methods
        .cancelMarket()
        .accountsStrict({
          market: marketAddress,
          marketCreator: hema.publicKey,
          higherPool: higherPoolAddress,
          lowerPool: lowerPoolAddress,
          creatorAta: creatorAta.address,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: token.TOKEN_PROGRAM_ID,
        })
        .signers([hema])
        .rpc()
        .then(confirmTx);

      const cancelledMarket = await program.account.market.fetchNullable(
        marketAddress
      );
      assert.isNull(cancelledMarket);

      const higherPool = await program.account.market.fetchNullable(
        higherPoolAddress
      );
      assert.isNull(higherPool);

      const lowerPool = await program.account.market.fetchNullable(
        lowerPoolAddress
      );
      assert.isNull(lowerPool);
    });
  });
  describe("Finalize Market", () => {
    it("Initializes market again", async () => {
      await airdrop(provider.connection, hema.publicKey);

      const [marketAddress, marketBump] = getMarketAddress(
        hema.publicKey,
        feedIdString,
        targetPrice,
        marketDuration,
        program.programId
      );

      await program.methods
        .initializeMarket(targetPrice, feedIdString, marketDuration)
        .accountsStrict({
          market: marketAddress,
          marketCreator: hema.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([hema])
        .rpc()
        .then(confirmTx);

      await checkMarket(
        program,
        marketAddress,
        hema.publicKey,
        feedIdString,
        targetPrice,
        marketDuration,
        marketBump,
        { initializedMarket: {} }
      );
    });

    it("Initialize pool again", async () => {
      await airdrop(provider.connection, hema.publicKey);
      await airdrop(provider.connection, mint_authority.publicKey);

      const mint = await token.createMint(
        provider.connection,
        mint_authority,
        mint_authority.publicKey,
        null,
        9
      );

      const userAta = await token.getOrCreateAssociatedTokenAccount(
        provider.connection,
        hema,
        mint,
        hema.publicKey
      );

      await token.mintTo(
        provider.connection,
        mint_authority,
        mint,
        userAta.address,
        mint_authority,
        to_mint.toNumber()
      );

      const [marketAddress, marketBump] = getMarketAddress(
        hema.publicKey,
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

      const [lowerPoolAddress, lowerPoolBump] = getPoolAddress(
        LOWER_POOL_SEED,
        marketAddress,
        program.programId
      );

      await program.methods
        .initializePools()
        .accountsStrict({
          market: marketAddress,
          marketCreator: hema.publicKey,
          poolTokenMint: mint,
          higherPool: higherPoolAddress,
          lowerPool: lowerPoolAddress,
          userAta: userAta.address,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: token.TOKEN_PROGRAM_ID,
        })
        .signers([hema])
        .rpc()
        .then(confirmTx);

      await checkMarket(
        program,
        marketAddress,
        hema.publicKey,
        feedIdString,
        targetPrice,
        marketDuration,
        marketBump,
        { initializedPools: {} },
        higherPoolBump,
        lowerPoolBump,
        mint
      );
    });

    it("Market Finalized", async () => {
      await airdrop(provider.connection, hema.publicKey);

      const [marketAddress, marketBump] = getMarketAddress(
        hema.publicKey,
        feedIdString,
        targetPrice,
        marketDuration,
        program.programId
      );
      const market = await program.account.market.fetch(marketAddress);

      const [higherPoolAddress, higherPoolBump] = getPoolAddress(
        HIGHER_POOL_SEED,
        marketAddress,
        program.programId
      );
      const [lowerPoolAddress, lowerPoolBump] = getPoolAddress(
        LOWER_POOL_SEED,
        marketAddress,
        program.programId
      );
      const creatorAta = await token.getOrCreateAssociatedTokenAccount(
        provider.connection,
        hema,
        market.mint,
        hema.publicKey
      );

      await checkMarket(
        program,
        marketAddress,
        hema.publicKey,
        feedIdString,
        targetPrice,
        marketDuration,
        marketBump,
        { initializedPools: {} },
        higherPoolBump,
        lowerPoolBump,
        market.mint
      );

      const creatorBalanceBefore = Number(creatorAta.amount);

      await program.methods
        .finalizeMarket()
        .accountsStrict({
          market: marketAddress,
          marketCreator: hema.publicKey,
          higherPool: higherPoolAddress,
          lowerPool: lowerPoolAddress,
          creatorAta: creatorAta.address,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: token.TOKEN_PROGRAM_ID,
        })
        .signers([hema])
        .rpc()
        .then(confirmTx);

      const creatorAtaAfter = await token.getAccount(
        provider.connection,
        creatorAta.address,
        "confirmed"
      );

      assert.strictEqual(
        Number(creatorAtaAfter.amount),
        creatorBalanceBefore + Number(INITIAL_USDC_AMOUNT) * 2
      );

      const cancelledMarket = await program.account.market.fetchNullable(
        marketAddress
      );
      assert.isNull(cancelledMarket);

      const higherPool = await program.account.market.fetchNullable(
        higherPoolAddress
      );
      assert.isNull(higherPool);

      const lowerPool = await program.account.market.fetchNullable(
        lowerPoolAddress
      );
      assert.isNull(lowerPool);
    });
  });
});

async function airdrop(
  connection: anchor.web3.Connection,
  address: PublicKey,
  amount = 10 * LAMPORTS_PER_SOL
) {
  await connection.requestAirdrop(address, amount).then(confirmTx);
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

function getBetAddress(
  marketAddress: PublicKey,
  userAddress: PublicKey,
  betAmount: anchor.BN,
  betDirection: Object,
  programId: PublicKey,
  betStringSeed = "prediction_bet"
) {
  const directionString = Object.keys(betDirection)[0];
  const directionNum = directionString == "higher" ? 0 : 1;
  const directionBN = new anchor.BN(directionNum);

  return PublicKey.findProgramAddressSync(
    [
      anchor.utils.bytes.utf8.encode(betStringSeed),
      userAddress.toBuffer(),
      marketAddress.toBuffer(),
      betAmount.toArrayLike(Buffer, "le", 8),
      Buffer.from(new Uint8Array([directionNum])),
    ],
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
  initialization: Object,
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

  const utf8ByteArray_content = stringToUtf8ByteArray(feedId);
  const paddedByteArray_content = padByteArrayWithZeroes(
    utf8ByteArray_content,
    66
  );
  assert.strictEqual(
    marketData.feedId.toString(),
    paddedByteArray_content.toString()
  );

  assert.strictEqual(
    Object.keys(marketData.initialization)[0],
    Object.keys(initialization)[0]
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

const confirmTx = async (signature: string) => {
  const latestBlockhash = await anchor
    .getProvider()
    .connection.getLatestBlockhash();
  await anchor.getProvider().connection.confirmTransaction(
    {
      signature,
      ...latestBlockhash,
    },
    "confirmed"
  );
};
