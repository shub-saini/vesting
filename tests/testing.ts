// import { Connection, Keypair, PublicKey } from '@solana/web3.js';
// import { fromWorkspace, LiteSVMProvider } from 'anchor-litesvm';
// import IDL from '../target/idl/vesting.json';
// import { describe, it } from 'mocha';
// import * as anchor from '@coral-xyz/anchor';
// import { Vesting } from '../target/types/vesting';
// import { createMint } from '@solana/spl-token';
// import { LiteSVM } from 'litesvm';

// const programId = new PublicKey(IDL.address);

// describe('Vesting', () => {
//   //   const wallet = new anchor.Wallet(Keypair.generate());
//   //   const client = fromWorkspace('.');
//   //   const provider = new LiteSVMProvider(client, wallet);
//   //   anchor.setProvider(provider);
//   const svm = new LiteSVM();

//   //   const connection = new Connection(svm.getEn)

//   //   const program = new anchor.Program<Vesting>(IDL, provider);

//   it('Test 1', async () => {
//     const mintAuthority = new Keypair();
//     const mint = await createMint(
//       provider.connection,
//       wallet.payer,
//       mintAuthority.publicKey,
//       null,
//       9
//     );

//     console.log(mint.toBase58());
//   });
// });
