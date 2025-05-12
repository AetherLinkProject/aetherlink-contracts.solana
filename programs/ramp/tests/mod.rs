// // ramp tests mod placeholder 

// #[cfg(feature = "test-bpf")]
// mod integration {
//     use super::*;
//     use secp256k1::{Secp256k1, SecretKey, Message, ecdsa::Signature};
//     use rand::rngs::OsRng;
//     // 这里可添加Anchor集成测试用例，如CPI调用、跨合约多签等
//     // #[tokio::test]
//     // async fn test_transmit_cpi() { ... }

//     #[tokio::test]
//     async fn test_initialize_integration() {
//         use anchor_lang::prelude::*;
//         use anchor_lang::solana_program::system_program;
//         use ramp::RampConfig;
//         use anchor_lang::InstructionData;
//         use ramp::accounts::Initialize;
//         use ramp::instruction::Initialize as InitializeIx;
//         use solana_sdk::signature::Keypair;
//         use solana_sdk::signer::Signer as _;
//         use solana_sdk::transaction::Transaction;
//         use solana_program_test::{processor, ProgramTest};
//         use solana_sdk::pubkey::Pubkey;
//         // 1. Create test context
//         let program_id = ramp::ID;
//         let program_test = ProgramTest::new(
//             "ramp",
//             program_id,
//             processor!(ramp::entry),
//         );
//         let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
//         // 2. Derive PDA for RampConfig
//         let (ramp_config_pda, _bump) = Pubkey::find_program_address(&[b"ramp_config"], &program_id);
//         // 3. Prepare accounts and instruction data
//         let config_data = [42u8; 32];
//         let authority = Keypair::new();
//         let accounts = Initialize {
//             ramp_config: ramp_config_pda,
//             user: payer.pubkey(),
//             system_program: system_program::ID,
//         };
//         let ix = solana_sdk::instruction::Instruction {
//             program_id,
//             accounts: accounts.to_account_metas(None),
//             data: InitializeIx { config_data, authority: authority.pubkey() }.data(),
//         };
//         // 4. Send transaction (Anchor will auto-create PDA)
//         let mut transaction = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
//         transaction.sign(&[&payer], recent_blockhash);
//         banks_client.process_transaction(transaction).await.unwrap();
//         // 5. Fetch and assert on-chain state
//         let ramp_config_account = banks_client.get_account(ramp_config_pda).await.unwrap().unwrap();
//         let ramp_config: RampConfig = RampConfig::try_deserialize(&mut ramp_config_account.data.as_ref()).unwrap();
//         assert!(ramp_config.is_initialized);
//         assert_eq!(ramp_config.authority, authority.pubkey());
//         assert_eq!(ramp_config.config_data, config_data);
//     }
// } 