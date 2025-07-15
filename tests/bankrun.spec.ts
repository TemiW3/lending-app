import { BanksClient, ProgramTestContext, startAnchor } from "solana-bankrun";
import IDL from "../target/idl/lending.json";
import { Lending } from "../target/types/lending";
import { describe, it } from "node:test";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { BankrunProvider } from "anchor-bankrun";
import { PythSolanaReceiver } from "@pythnetwork/pyth-solana-receiver";
import { BankrunContextWrapper } from "../bankrun-utils/bankrunConnection";
import { BN, Program } from "@coral-xyz/anchor";
import { createMint, mintTo, createAccount } from "spl-token-bankrun";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

describe("Lending smart contract test", async () => {
  let context: ProgramTestContext;
  let provider: BankrunProvider;
  let bankrunContextWrapper: BankrunContextWrapper;
  let program: Program<Lending>;
  let banksClient: BanksClient;
  let signer: Keypair;
  let usdcBankAccount: PublicKey;
  let solBankAccount: PublicKey;

  const pyth = new PublicKey("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");
  const devnetConnection = new Connection("https://api.devnet.solana.com");
  const accountInfo = await devnetConnection.getAccountInfo(pyth);
  context = await startAnchor(
    "",
    [{ name: "lending", programId: new PublicKey(IDL.address) }],
    [{ address: pyth, info: accountInfo }]
  );
  provider = new BankrunProvider(context);

  const SOL_PRICE_FEED_ID =
    "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";

  bankrunContextWrapper = new BankrunContextWrapper(context);

  const connection = bankrunContextWrapper.connection.toConnection();

  const pythSolanaReceiver = new PythSolanaReceiver({
    connection,
    wallet: provider.wallet,
  });

  const solUsdPriceFeedAccount = pythSolanaReceiver.getPriceFeedAccountAddress(
    0,
    SOL_PRICE_FEED_ID
  );

  const feedAccountInfo = await devnetConnection.getAccountInfo(
    solUsdPriceFeedAccount
  );

  context.setAccount(solUsdPriceFeedAccount, feedAccountInfo);

  program = new Program<Lending>(IDL as Lending, provider);
  banksClient = context.banksClient;
  signer = provider.wallet.payer;

  const mintUsdc = await createMint(
    banksClient,
    signer,
    signer.publicKey,
    null,
    2
  );

  const mintSol = await createMint(
    banksClient,
    signer,
    signer.publicKey,
    null,
    2
  );

  [usdcBankAccount] = PublicKey.findProgramAddressSync(
    [Buffer.from("treasury", mintUsdc.toBuffer())],
    program.programId
  );

  [solBankAccount] = PublicKey.findProgramAddressSync(
    [Buffer.from("treasury", mintSol.toBuffer())],
    program.programId
  );

  it("Test init and fund bank", async () => {
    const initUsdcBankTx = await program.methods
      .initializeBank(new BN(1), new BN(1))
      .accounts({
        signer: signer.publicKey,
        mint: mintUsdc,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc({ commitment: "confirmed" });

    console.log("Created the USDC bank account:", initUsdcBankTx);

    const amount = 10_000 * 10 ** 9;

    const mintTx = await mintTo(
      banksClient,
      signer,
      mintUsdc,
      usdcBankAccount,
      signer,
      amount
    );

    console.log("Mint USDC to bank: ", mintTx);
  });

  it("Test Init user ", async () => {
    const initUserTx = program.methods
      .initializeUser(mintUsdc)
      .accounts({
        signer: signer.publicKey,
      })
      .rpc({ commitment: "confirmed" });

    console.log("Init user:", initUserTx);
  });

  it("Test init and fund sol bank", async () => {
    const initSolBankTx = await program.methods
      .initializeBank(new BN(2), new BN(1))
      .accounts({
        signer: signer.publicKey,
        mint: mintSol,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc({ commitment: "confirmed" });

    console.log("created SOL Bank account:", initSolBankTx);

    const amount = 10_000 * 10 ** 9;

    const mintTx = await mintTo(
      banksClient,
      signer,
      mintSol,
      solBankAccount,
      signer,
      amount
    );

    console.log("Mint SOL to bank: ", mintTx);
  });

  it("Create and Fund Token Account", async () => {
    const usdcTokenAccount = await createAccount(
      // @ts-ignores
      banksClient,
      signer,
      mintUsdc,
      signer.publicKey
    );

    console.log("USDC Token Account Created:", usdcTokenAccount);

    const amount = 10_000 * 10 ** 9;
    const mintUSDCTx = await mintTo(
      // @ts-ignores
      banksClient,
      signer,
      mintUsdc,
      usdcTokenAccount,
      signer,
      amount
    );

    console.log("Mint to USDC Bank Signature:", mintUSDCTx);
  });
});
