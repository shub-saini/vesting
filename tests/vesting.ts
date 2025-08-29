import * as anchor from '@coral-xyz/anchor';
import { Program, BN } from '@coral-xyz/anchor';
import type { Vesting } from '../target/types/vesting';
import { BankrunProvider } from 'anchor-bankrun';
import {
  BanksClient,
  Clock,
  ProgramTestContext,
  startAnchor,
} from 'solana-bankrun';
import { Keypair, LAMPORTS_PER_SOL, PublicKey } from '@solana/web3.js';
import IDL from '../target/idl/vesting.json';
import { SYSTEM_PROGRAM_ID } from '@coral-xyz/anchor/dist/cjs/native/system';
import {
  createAssociatedTokenAccount,
  createMint,
  mintTo,
} from 'spl-token-bankrun';
import { TOKEN_PROGRAM_ID } from '@coral-xyz/anchor/dist/cjs/utils/token';
import { expect } from 'chai';
import {
  AccountLayout,
  getAssociatedTokenAddressSync,
} from '@solana/spl-token';
import NodeWallet from '@coral-xyz/anchor/dist/cjs/nodewallet';

describe('vesting', () => {
  const U64_BYTES = 8;
  const COMPANY_NAME = 'company';
  const VESTING_START_TIME = 0;
  const VESTING_CLIFF_TIME = 400;
  const VESTING_END_TIME = 1000;
  const VESTING_MIDDLE_TIME_AFTER_CLIFF = 500;
  const VESTING_ACCOUNT_ID = new BN(1);
  const ASSIGNED_AMOUNT_TO_BENEFICIARY = new BN(10);

  let provider: BankrunProvider;
  let context: ProgramTestContext;
  let programId: PublicKey;
  let beneficiary: Keypair;
  let program: Program<Vesting>;
  let banksClient: BanksClient;
  let employer: Keypair;
  let mint: PublicKey;
  let employer_ata: PublicKey;
  let beneficiaryProgram: Program<Vesting>;
  let beneficiaryProvider: BankrunProvider;
  const decimals = 9;
  const LAMPORTS_PER_MINT_TOKEN = new BN(10 ** decimals);
  const TOKEN_FUNDED_AMOUNT = new BN(100);
  let vestingAccount: PublicKey;
  let treasuryTokenAccount: PublicKey;
  let beneficiaryVestingAccount: PublicKey;

  anchor.setProvider(anchor.AnchorProvider.env());

  before(async () => {
    programId = new PublicKey(IDL.address);
    beneficiary = new Keypair();

    context = await startAnchor(
      '',
      [{ name: 'vesting', programId }],
      [
        {
          address: beneficiary.publicKey,
          info: {
            lamports: 1 * LAMPORTS_PER_SOL,
            data: Buffer.alloc(0),
            owner: SYSTEM_PROGRAM_ID,
            executable: false,
          },
        },
      ]
    );
    provider = new BankrunProvider(context);
    anchor.setProvider(provider);
    // program = anchor.workspace.vesting as Program<Vesting>;
    program = new Program<Vesting>(IDL as Vesting, provider);
    banksClient = context.banksClient;
    employer = provider.wallet.payer;

    beneficiaryProvider = new BankrunProvider(context);
    beneficiaryProvider.wallet = new NodeWallet(beneficiary);
    beneficiaryProgram = new Program<Vesting>(
      IDL as Vesting,
      beneficiaryProvider
    );

    mint = await createMint(
      // @ts-ignore
      banksClient,
      employer,
      employer.publicKey,
      null,
      9
    );

    employer_ata = await createAssociatedTokenAccount(
      // @ts-ignore
      banksClient,
      employer,
      mint,
      employer.publicKey
    );

    await mintTo(
      // @ts-ignore
      banksClient,
      employer,
      mint,
      employer_ata,
      employer,
      TOKEN_FUNDED_AMOUNT.mul(LAMPORTS_PER_MINT_TOKEN)
    );

    const idBuf = Buffer.alloc(U64_BYTES);
    idBuf.writeBigUInt64LE(BigInt(VESTING_ACCOUNT_ID.toString()));

    [vestingAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from('vesting_account'), Buffer.from(COMPANY_NAME), idBuf],
      programId
    );

    [treasuryTokenAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from('vesting_treasury'), vestingAccount.toBuffer()],
      programId
    );

    [beneficiaryVestingAccount] = PublicKey.findProgramAddressSync(
      [
        Buffer.from('beneficiary_vesting_account'),
        beneficiary.publicKey.toBuffer(),
        vestingAccount.toBuffer(),
      ],
      programId
    );
  });

  it('Create Vesting account', async () => {
    await program.methods
      .createVestingAccount(new BN(VESTING_ACCOUNT_ID), COMPANY_NAME)
      .accounts({
        mint: mint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc({ commitment: 'confirmed', skipPreflight: false });

    const vestingAccountData = await program.account.vestingAccount.fetch(
      vestingAccount,
      'confirmed'
    );

    expect(vestingAccountData.id.toString()).equal(
      VESTING_ACCOUNT_ID.toString()
    );
    expect(vestingAccountData.companyName).equal(COMPANY_NAME);
    expect(vestingAccountData.mint.toBase58()).equal(mint.toBase58());
  });

  it('Transfering token to vesting treasury', async () => {
    await program.methods
      .transferTokensToTreasury(
        TOKEN_FUNDED_AMOUNT.mul(LAMPORTS_PER_MINT_TOKEN)
      )
      .accounts({
        funder: employer.publicKey,
        mint,
        vestingAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc({ commitment: 'confirmed', skipPreflight: false });

    const accountInfo = await banksClient.getAccount(treasuryTokenAccount);
    const tokenAccountData = AccountLayout.decode(accountInfo!.data);

    expect(tokenAccountData.amount.toString()).equal(
      TOKEN_FUNDED_AMOUNT.mul(LAMPORTS_PER_MINT_TOKEN).toString()
    );
  });

  it('Transfering token to vesting treasury fails if invalid Mint', async () => {
    const tempMint = await createMint(
      // @ts-ignore
      banksClient,
      employer,
      employer.publicKey,
      null,
      decimals
    );

    await createAssociatedTokenAccount(
      // @ts-ignore
      banksClient,
      employer,
      tempMint,
      employer.publicKey
    );

    try {
      await program.methods
        .transferTokensToTreasury(
          TOKEN_FUNDED_AMOUNT.mul(LAMPORTS_PER_MINT_TOKEN)
        )
        .accounts({
          funder: employer.publicKey,
          mint: tempMint,
          vestingAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc({ commitment: 'confirmed', skipPreflight: false });
    } catch (error) {
      expect(error.toString()).to.includes('InvalidMint');
    }
  });

  it('Initialize Vesting Schedule for beneficiary', async () => {
    const startTime = new BN(VESTING_START_TIME);
    const endTime = new BN(VESTING_END_TIME);
    const totalAmount = new BN(ASSIGNED_AMOUNT_TO_BENEFICIARY).mul(
      LAMPORTS_PER_MINT_TOKEN
    );
    const cliffTime = new BN(VESTING_CLIFF_TIME);

    await program.methods
      .initializeVestingSchedule(startTime, endTime, totalAmount, cliffTime)
      .accounts({
        vestingAccount,
        mint,
        beneficiary: beneficiary.publicKey,
      })
      .rpc({ commitment: 'confirmed', skipPreflight: false });

    const vestingScheduleData = await program.account.beneficiaryAccount.fetch(
      beneficiaryVestingAccount
    );

    expect(vestingScheduleData.beneficiary.toBase58()).equal(
      beneficiary.publicKey.toBase58()
    );
  });

  it('Initialize Vesting fails if total_amount is less than 1', async () => {
    const tempAccount = new Keypair();
    const startTime = new BN(VESTING_START_TIME);
    const endTime = new BN(VESTING_END_TIME);
    const totalAmount = new BN(0);
    const cliffTime = new BN(VESTING_CLIFF_TIME);

    try {
      await program.methods
        .initializeVestingSchedule(startTime, endTime, totalAmount, cliffTime)
        .accounts({
          vestingAccount,
          mint,
          beneficiary: tempAccount.publicKey,
        })
        .rpc({ commitment: 'confirmed', skipPreflight: false });
    } catch (error) {
      expect(error.toString()).to.include('VestingAmountShoulBePositive');
    }
  });

  it('Vesting Schedule Initialization fails if treasury doesnt have enough money', async () => {
    const tempAccount = new Keypair();
    const startTime = new BN(VESTING_START_TIME);
    const endTime = new BN(VESTING_END_TIME);
    const totalAmount = LAMPORTS_PER_MINT_TOKEN.mul(LAMPORTS_PER_MINT_TOKEN);
    const cliffTime = new BN(VESTING_CLIFF_TIME);

    try {
      await program.methods
        .initializeVestingSchedule(startTime, endTime, totalAmount, cliffTime)
        .accounts({
          vestingAccount,
          mint,
          beneficiary: tempAccount.publicKey,
        })
        .rpc({ commitment: 'confirmed', skipPreflight: false });
    } catch (error) {
      expect(error.toString()).to.include('NotEnoughTokensInTreasury');
    }
  });

  it('should fail when time constraints violated', async () => {
    const tempAccount = new Keypair();
    const startTime = new BN(VESTING_START_TIME);
    const endTime = new BN(VESTING_START_TIME);
    const totalAmount = new BN(ASSIGNED_AMOUNT_TO_BENEFICIARY).mul(
      LAMPORTS_PER_MINT_TOKEN
    );
    const cliffTime = new BN(VESTING_CLIFF_TIME);
    try {
      await program.methods
        .initializeVestingSchedule(startTime, endTime, totalAmount, cliffTime)
        .accounts({
          vestingAccount,
          mint,
          beneficiary: tempAccount.publicKey,
        })
        .rpc({ commitment: 'confirmed', skipPreflight: false });
    } catch (error) {
      expect(error.toString()).to.include('InvalidVestingSchedule');
    }
  });

  it('admin can change admin', async () => {
    const temporaryAdmin = beneficiary;
    await program.methods
      .changeAdmin()
      .accounts({
        vestingAccount,
        newAdmin: temporaryAdmin.publicKey,
      })
      .rpc({ commitment: 'confirmed', skipPreflight: true });

    let admin = (await program.account.vestingAccount.fetch(vestingAccount))
      .admin;

    expect(admin.toBase58()).equal(temporaryAdmin.publicKey.toBase58());

    await beneficiaryProgram.methods
      .changeAdmin()
      .accounts({
        admin: temporaryAdmin.publicKey,
        vestingAccount,
        newAdmin: employer.publicKey,
      })
      .signers([temporaryAdmin])
      .rpc({ commitment: 'confirmed', skipPreflight: true });

    admin = (await program.account.vestingAccount.fetch(vestingAccount)).admin;

    expect(admin.toBase58()).equal(employer.publicKey.toBase58());
  });

  it('Admin Change fail if not done by admin', async () => {
    const tempAccount = new Keypair();
    try {
      await beneficiaryProgram.methods
        .changeAdmin()
        .accounts({
          admin: beneficiary.publicKey,
          vestingAccount,
          newAdmin: tempAccount.publicKey,
        })
        .signers([beneficiary])
        .rpc({ commitment: 'confirmed', skipPreflight: true });
    } catch (error) {
      expect(error.toString()).to.includes('UnAuthorized');
    }
  });

  it('user can claim tokens linearly', async () => {
    const currentClock = await banksClient.getClock();
    context.setClock(
      new Clock(
        currentClock.slot,
        currentClock.epochStartTimestamp,
        currentClock.epoch,
        currentClock.leaderScheduleEpoch,
        BigInt(VESTING_MIDDLE_TIME_AFTER_CLIFF)
      )
    );

    await beneficiaryProgram.methods
      .claimVestedTokens(COMPANY_NAME, VESTING_ACCOUNT_ID)
      .accounts({
        beneficiary: beneficiary.publicKey,
        mint,
        vestingAccount,
        treasuryTokenAccount,
        beneficiaryVestingAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([beneficiary])
      .rpc({ commitment: 'confirmed', skipPreflight: true });

    const beneficiary_ata = getAssociatedTokenAddressSync(
      mint,
      beneficiary.publicKey,
      false
    );

    const accountInfo = await banksClient.getAccount(beneficiary_ata);
    const tokenAccountData = AccountLayout.decode(accountInfo!.data);

    expect(tokenAccountData.amount.toString()).equal(
      ASSIGNED_AMOUNT_TO_BENEFICIARY.mul(LAMPORTS_PER_MINT_TOKEN)
        .mul(new BN(VESTING_MIDDLE_TIME_AFTER_CLIFF))
        .div(new BN(VESTING_END_TIME))
        .toString()
    );
  });

  it('token claims fails when there is nothing to claim', async () => {
    try {
      await beneficiaryProgram.methods
        .claimVestedTokens(COMPANY_NAME, VESTING_ACCOUNT_ID)
        .accounts({
          beneficiary: beneficiary.publicKey,
          mint,
          vestingAccount,
          treasuryTokenAccount,
          beneficiaryVestingAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([beneficiary])
        .rpc({ commitment: 'confirmed', skipPreflight: true });
    } catch (error) {
      expect(error.toString()).to.includes('NothingToClaim');
    }
  });

  it('Admin can revoke a beneficiary Vesting', async () => {
    const clock = await banksClient.getClock();

    await program.methods
      .revokeBeneficiaryAccount()
      .accounts({
        beneficiary: beneficiary.publicKey,
        vestingAccount,
      })
      .rpc();

    const beneficiaryVestingAccountData =
      await program.account.beneficiaryAccount.fetch(beneficiaryVestingAccount);

    expect(beneficiaryVestingAccountData.revokeAt.toString()).equal(
      clock.unixTimestamp.toString()
    );
  });

  it('Beneficiary claiming fails after revoking, cant claim anything after that point', async () => {
    const currentClock = await banksClient.getClock();
    context.setClock(
      new Clock(
        currentClock.slot,
        currentClock.epochStartTimestamp,
        currentClock.epoch,
        currentClock.leaderScheduleEpoch,
        BigInt(VESTING_END_TIME)
      )
    );

    try {
      await beneficiaryProgram.methods
        .claimVestedTokens(COMPANY_NAME, VESTING_ACCOUNT_ID)
        .accounts({
          beneficiary: beneficiary.publicKey,
          mint,
          vestingAccount,
          treasuryTokenAccount,
          beneficiaryVestingAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([beneficiary])
        .rpc({ commitment: 'confirmed', skipPreflight: true });
    } catch (error) {
      expect(error.toString()).to.includes('NothingToClaim');
    }
  });
});
