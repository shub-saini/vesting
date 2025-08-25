import * as anchor from '@coral-xyz/anchor';
import { Program, BN } from '@coral-xyz/anchor';
import type { Vesting } from '../target/types/vesting';
import { BankrunProvider } from 'anchor-bankrun';
import { BanksClient, ProgramTestContext, startAnchor } from 'solana-bankrun';
import { Keypair, LAMPORTS_PER_SOL, PublicKey } from '@solana/web3.js';
import IDL from '../target/idl/vesting.json';
import { SYSTEM_PROGRAM_ID } from '@coral-xyz/anchor/dist/cjs/native/system';
import { createMint } from 'spl-token-bankrun';
import { TOKEN_PROGRAM_ID } from '@coral-xyz/anchor/dist/cjs/utils/token';
import { expect } from 'chai';

describe('vesting', () => {
  const companyName = 'company';
  let provider: BankrunProvider;
  let context: ProgramTestContext;
  let programId: PublicKey;
  let beneficiary: Keypair;
  let program: Program<Vesting>;
  let banksClient: BanksClient;
  let employer: Keypair;
  let mint: PublicKey;
  let vestingAccount: PublicKey;
  let vestingAccountId: BN;
  let treasuryTokenAccount: PublicKey;
  let vestingScheduleAccount: PublicKey;

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
    program = anchor.workspace.vesting as Program<Vesting>;
    // program = new Program<Vesting>(IDL as Vesting, provider);
    banksClient = context.banksClient;
    employer = provider.wallet.payer;

    mint = await createMint(
      // @ts-ignore
      banksClient,
      employer,
      employer.publicKey,
      null,
      9
    );

    vestingAccountId = new BN(1);
    const idBuf = Buffer.alloc(8);
    idBuf.writeBigUInt64LE(BigInt(vestingAccountId.toString()));

    [vestingAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from('vesting_account'), Buffer.from(companyName), idBuf],
      programId
    );

    [treasuryTokenAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from('vesting_treasury'), vestingAccount.toBuffer()],
      programId
    );

    [vestingScheduleAccount] = PublicKey.findProgramAddressSync(
      [
        Buffer.from('beneficiary_vesting_schedule'),
        beneficiary.publicKey.toBuffer(),
        vestingAccount.toBuffer(),
      ],
      programId
    );
  });

  it('Create Vesting account', async () => {
    await program.methods
      .createVestingAccount(new BN(vestingAccountId), companyName)
      .accounts({
        mint: mint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc({ commitment: 'confirmed', skipPreflight: false });

    const vestingAccountData = await program.account.vestingAccount.fetch(
      vestingAccount,
      'confirmed'
    );

    expect(vestingAccountData.id.toString()).equal(vestingAccountId.toString());
    expect(vestingAccountData.companyName).equal(companyName);
    expect(vestingAccountData.mint.toBase58()).equal(mint.toBase58());
  });

  it('Initialize Vesting Schedule for beneficiary', async () => {
    const startTime = new BN(0);
    const endTime = new BN(1000);
    const totalAmount = new BN(10 * 1_000_000_000);
    const cliffTime = new BN(400);

    await program.methods
      .initializeVestingSchedule(startTime, endTime, totalAmount, cliffTime)
      .accounts({
        vestingAccount,
        mint,
        beneficiary: beneficiary.publicKey,
      })
      .rpc({ commitment: 'confirmed', skipPreflight: false });

    const vestingScheduleData = await program.account.beneficiaryAccount.fetch(
      vestingScheduleAccount
    );

    expect(vestingScheduleData.beneficiary.toBase58()).equal(
      beneficiary.publicKey.toBase58()
    );
  });

  it('should fail when time constraints violated', async () => {
    const tempAccount = new Keypair();
    const startTime = new BN(1000);
    const endTime = new BN(1000);
    const totalAmount = new BN(10 * 1_000_000_000);
    const cliffTime = new BN(400);
    try {
      await program.methods
        .initializeVestingSchedule(startTime, endTime, totalAmount, cliffTime)
        .accounts({
          vestingAccount,
          mint,
          beneficiary: tempAccount.publicKey,
        })
        .rpc({ commitment: 'confirmed', skipPreflight: false });

      expect.fail('Time constraints not satisfied');
    } catch (error) {
      console.log(error);
      expect(error.toString()).to.include('InvalidVestingSchedule');
    }
  });
});
