use program_test::*;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer, transport::TransportError};

mod program_test;

#[allow(unaligned_references)]
#[tokio::test]
async fn test_voting() -> Result<(), TransportError> {
    let context = TestContext::new().await;
    let addin = &context.addin;

    let payer = &context.users[0].key;
    let realm_authority = Keypair::new();
    let realm = context
        .governance
        .create_realm(
            "testrealm",
            realm_authority.pubkey(),
            &context.mints[0],
            &payer,
            &context.addin.program_id,
        )
        .await;

    let voter_authority = &context.users[1].key;
    let voter2_authority = &context.users[2].key;
    let token_owner_record = realm
        .create_token_owner_record(voter_authority.pubkey(), &payer)
        .await;
    let token_owner_record2 = realm
        .create_token_owner_record(voter2_authority.pubkey(), &payer)
        .await;

    let registrar = addin
        .create_registrar(&realm, &realm_authority, payer)
        .await;

    let voter = addin
        .create_voter_weight_record(&registrar, &token_owner_record, &voter_authority, &payer)
        .await;
    let voter2 = addin
        .create_voter_weight_record(&registrar, &token_owner_record2, &voter2_authority, &payer)
        .await;
    
    let mint_governance = realm
        .create_mint_governance(
            context.mints[0].pubkey.unwrap(),
            &context.mints[0].authority,
            &voter,
            &voter_authority,
            payer,
            addin.update_voter_weight_record_instruction(&registrar, &voter),
        )
        .await;

    realm
        .create_proposal(
            mint_governance.address,
            &voter_authority,
            &voter,
            payer,
            addin.update_voter_weight_record_instruction(&registrar, &voter),
        )
        .await
        .expect_err("not enough tokens to create proposal");
    
    context.solana.advance_clock_by_slots(2).await; // avoid cache when sending same transaction again

    let proposal = realm
        .create_proposal(
            mint_governance.address,
            &voter_authority,
            &voter,
            payer,
            addin.update_voter_weight_record_instruction(&registrar, &voter),
        )
        .await
        .unwrap();

    realm
        .cast_vote(
            mint_governance.address,
            &proposal,
            &voter2,
            &voter2_authority,
            payer,
            addin.update_voter_weight_record_instruction(&registrar, &voter2),
        )
        .await
        .unwrap();

    let proposal_data = context.solana.get_account_data(proposal.address).await;
    let mut data_slice: &[u8] = &proposal_data;
    let proposal_state: spl_governance::state::proposal::ProposalV2 =
        anchor_lang::AnchorDeserialize::deserialize(&mut data_slice).unwrap();
    assert_eq!(proposal_state.options[0].vote_weight, 2 * 750);
    assert_eq!(proposal_state.deny_vote_weight.unwrap(), 0);

    Ok(())
}
