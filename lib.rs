#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod votaciones {
    use ink::prelude::string::String;
    use ink::storage::Mapping;
    use scale::{Decode, Encode};
    #[cfg(feature = "std")]
    use ink::storage::traits::StorageLayout;
    #[cfg(feature = "std")]
    use scale_info::TypeInfo;



    #[derive(Encode, Decode, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(TypeInfo))]
    pub enum Error {
        NotOwner,
        ProposalNotFound,
        AlreadyVoted,
        MaxProposalsReached,
        Overflow,
    }




#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(TypeInfo, StorageLayout))]
pub struct Proposal {
    id: u32,
    title: String,
    votes_for: u32,
    votes_against: u32,
}



    #[ink(event)]
    pub struct ProposalCreated {
        #[ink(topic)]
        id: u32,
        title: String,
    }
    
    #[ink(event)]
    pub struct VoteCast {
        #[ink(topic)]
        proposalid: u32,
        #[ink(topic)]
        voter: AccountId,
        state: bool,
    }


    #[ink(storage)]
    pub struct Votaciones {
        owner: AccountId,
        proposals: Mapping<u32, Proposal>,
        has_voted: Mapping<(u32, AccountId), bool>,
        next_proposal_id: u32,
    }

    impl Votaciones {
        /// Constructor that initializes the owner with the caller address.
       #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                owner: Self::env().caller(),
                proposals: Mapping::default(),
                has_voted: Mapping::default(),
                next_proposal_id: 0,
            }
}


        /// Creates a new proposal. Only the owner can create proposals.
        /// The next proposal Id is incremented after using it on the new proposal.
        /// The new proposal is inserted in the mapping of the contract

       #[ink(message)]
        pub fn create_proposal(&mut self, title: String) -> Result<u32, Error> {
            let caller = self.env().caller();
            if caller != self.owner {
                return Err(Error::NotOwner);
            }

            let id = self.next_proposal_id;
            self.next_proposal_id = self
                .next_proposal_id
                .checked_add(1)
                .ok_or(Error::MaxProposalsReached)?;
           

            let proposal = Proposal {
                id,
                title: title.clone(),
                votes_for: 0,
                votes_against: 0,
            };

            self.proposals.insert(id, &proposal);
            self.env().emit_event(ProposalCreated { id, title });
            Ok(id)
        }

        /// Casts a vote on a proposal identified by `proposalid`.
        /// The vote is reverted if the address has already voted on that proposal id.
        #[ink(message)]
    pub fn vote(&mut self, proposalid: u32, state: bool) -> Result<(), Error> {
        let caller = self.env().caller();

        // existe proposal?
        let mut proposal = self.proposals.get(proposalid).ok_or(Error::ProposalNotFound)?;

        // ya votó?
        let key = (proposalid, caller);
        if self.has_voted.get(key).unwrap_or(false) {
            return Err(Error::AlreadyVoted);
        }

        // contar voto
        if state {
            proposal.votes_for = proposal
                .votes_for
                .checked_add(1)
                .ok_or(Error::Overflow)?;
        } else {
            proposal.votes_against = proposal
                .votes_against
                .checked_add(1)
                .ok_or(Error::Overflow)?;
        }

        // inserts updates
        self.proposals.insert(proposalid, &proposal);
        self.has_voted.insert(key, &true);

        self.env().emit_event(VoteCast { proposalid, voter: caller, state });
        Ok(())
    }


        /// Gets the proposal from the storage of the contract using the id
    #[ink(message)]
        pub fn get_proposal(&self, proposal_id: u32) -> Result<(String, u32, u32), Error> {
            let proposal = self.proposals.get(proposal_id).ok_or(Error::ProposalNotFound)?;
            Ok((proposal.title.clone(), proposal.votes_for, proposal.votes_against))
    }

        /// Returns the total number of proposals created
       #[ink(message)]
        pub fn total_proposals(&self) -> u32 {
            self.next_proposal_id
        }




    }

    /// Testing on this contract includes: unit tests and end-to-end tests.
    /// In which the tested scenarios include:
    /// - Proposal creation by the owner
    /// - Proposal creation by a non-owner (should fail)
    /// - Voting on a proposal
    /// - Voting twice on the same proposal (should fail)
    /// - Voting on a non-existent proposal (should fail)
    #[cfg(test)]
    mod tests {
            use super::*;
            use ink::env::test;
            use ink::env::DefaultEnvironment;

            fn default_accounts() -> test::DefaultAccounts<DefaultEnvironment> {
                test::default_accounts::<DefaultEnvironment>()
            }

            fn set_caller(caller: AccountId) {
                test::set_caller::<DefaultEnvironment>(caller);
            }

            fn set_callee(callee: AccountId) {
                test::set_callee::<DefaultEnvironment>(callee);
            }

            #[test]
            fn owner_can_create_proposal() {
                let accounts = default_accounts();
                set_caller(accounts.alice);
                set_callee(accounts.charlie);

                let mut contract = Votaciones::new();

                let result = contract.create_proposal("Titulo".to_string());
                assert_eq!(result, Ok(0));

                let proposal = contract.get_proposal(0).expect("proposal stored");
                assert_eq!(proposal, ("Titulo".to_string(), 0, 0));
                assert_eq!(contract.total_proposals(), 1);

                let events: Vec<test::EmittedEvent> = test::recorded_events().collect();
                assert_eq!(events.len(), 1);
                let decoded = <ProposalCreated as scale::Decode>::decode(&mut &events[0].data[..])
                    .expect("decode event");
                assert_eq!(decoded.id, 0);
                assert_eq!(decoded.title, "Titulo");
            }

            #[test]
            fn non_owner_cannot_create_proposal() {
                let accounts = default_accounts();
                set_caller(accounts.alice);
                set_callee(accounts.charlie);
                let mut contract = Votaciones::new();

                set_caller(accounts.bob);
                let result = contract.create_proposal("Tema".to_string());
                assert_eq!(result, Err(Error::NotOwner));
            }

            #[test]
            fn vote_records_support() {
                let accounts = default_accounts();
                set_caller(accounts.alice);
                set_callee(accounts.charlie);
                let mut contract = Votaciones::new();
                let proposal_id = contract.create_proposal("Tema".to_string()).unwrap();

                set_caller(accounts.bob);
                let outcome = contract.vote(proposal_id, true);
                assert_eq!(outcome, Ok(()));

                let (_, votes_for, votes_against) = contract.get_proposal(proposal_id).unwrap();
                assert_eq!(votes_for, 1);
                assert_eq!(votes_against, 0);

                let events: Vec<test::EmittedEvent> = test::recorded_events().collect();
                let decoded = <VoteCast as scale::Decode>::decode(&mut &events.last().unwrap().data[..])
                    .expect("decode event");
                assert_eq!(decoded.proposalid, proposal_id);
                assert_eq!(decoded.voter, accounts.bob);
                assert!(decoded.state);
            }

            #[test]
            fn voting_twice_reverts() {
                let accounts = default_accounts();
                set_caller(accounts.alice);
                set_callee(accounts.charlie);
                let mut contract = Votaciones::new();
                let proposal_id = contract.create_proposal("Tema".to_string()).unwrap();

                set_caller(accounts.bob);
                assert_eq!(contract.vote(proposal_id, true), Ok(()));
                let second = contract.vote(proposal_id, true);
                assert_eq!(second, Err(Error::AlreadyVoted));

                let (_, votes_for, votes_against) = contract.get_proposal(proposal_id).unwrap();
                assert_eq!(votes_for, 1);
                assert_eq!(votes_against, 0);
            }

            #[test]
            fn voting_nonexistent_reverts() {
                let accounts = default_accounts();
                set_caller(accounts.alice);
                set_callee(accounts.charlie);
                let mut contract = Votaciones::new();

                set_caller(accounts.bob);
                let result = contract.vote(42, true);
                assert_eq!(result, Err(Error::ProposalNotFound));
            }

    }


    /// e2e tests
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn e2e_create_and_vote(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let alice = ink_e2e::alice();
            let bob = ink_e2e::bob();

            let mut constructor = VotacionesRef::new();
            let contract = client
                .instantiate("votaciones", &alice, &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");

            let mut contract_ref = contract.call_builder::<Votaciones>();

            let create = contract_ref.create_proposal("Tema".into());
            let mut create_call = client.call(&alice, &create);
            let create_outcome = create_call.submit().await?;
            let proposal_id = create_outcome.return_value();
            assert_eq!(proposal_id, Ok(0));

            let vote = contract_ref.vote(0, true);
            let mut vote_call = client.call(&bob, &vote);
            let vote_outcome = vote_call.submit().await?;
            assert_eq!(vote_outcome.return_value(), Ok(()));

            let get = contract_ref.get_proposal(0);
            let mut get_call = client.call(&alice, &get);
            let get_outcome = get_call.dry_run().await?;
            let (title, votes_for, votes_against) = get_outcome.return_value().unwrap();
            assert_eq!(title, "Tema");
            assert_eq!(votes_for, 1);
            assert_eq!(votes_against, 0);

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_double_vote_reverts(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let alice = ink_e2e::alice();
            let bob = ink_e2e::bob();

            let mut constructor = VotacionesRef::new();
            let contract = client
                .instantiate("votaciones", &alice, &mut constructor)
                .submit()
                .await?;

            let mut contract_ref = contract.call_builder::<Votaciones>();

            let create = contract_ref.create_proposal("Tema".into());
            let mut create_call = client.call(&alice, &create);
            create_call.submit().await?;

            let vote = contract_ref.vote(0, true);
            let mut vote_call = client.call(&bob, &vote);
            vote_call.submit().await?;

            let mut second_call = client.call(&bob, &vote);
            let second_vote = second_call.dry_run().await?.return_value();
            assert_eq!(second_vote, Err(Error::AlreadyVoted));

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_vote_nonexistent_reverts(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let alice = ink_e2e::alice();
            let bob = ink_e2e::bob();

            let mut constructor = VotacionesRef::new();
            let contract = client
                .instantiate("votaciones", &alice, &mut constructor)
                .submit()
                .await?;

            let mut contract_ref = contract.call_builder::<Votaciones>();

            let vote = contract_ref.vote(99, true);
            let mut vote_call = client.call(&bob, &vote);
            let result = vote_call.dry_run().await?.return_value();
            assert_eq!(result, Err(Error::ProposalNotFound));

            Ok(())
        }
    }
}
