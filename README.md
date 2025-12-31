# On-Chain Voting Smart Contract

A decentralized proposal and voting smart contract written in **Rust** using the **ink!** framework for Substrate-based blockchains.

This project implements a simple yet robust governance mechanism where an owner creates proposals and participants can vote once per proposal, with all state changes recorded on-chain.

---

## Overview

The contract enables:
- Secure creation of proposals by a designated owner
- One-vote-per-account enforcement per proposal
- Transparent and immutable vote counting
- Event emission for proposal creation and vote casting

It is designed as a lightweight governance primitive suitable for DAOs, on-chain decision systems, or educational purposes.

---

## Features

- **Owner-controlled proposal creation**
- **One vote per account per proposal**
- **Overflow-safe vote counting**
- **On-chain event emission**
- **Explicit error handling**
- **Comprehensive unit tests**

---

## Tech Stack

- **Rust**
- **ink!** (Substrate smart contract framework)

---

## Contract Structure

### Storage
- `owner`: Contract owner account
- `proposals`: Mapping of proposal IDs to proposal data
- `has_voted`: Mapping to track whether an account has voted on a proposal
- `next_proposal_id`: Auto-incremented proposal counter

### Proposal Model
Each proposal contains:
- Unique ID
- Title
- Votes in favor
- Votes against

---

## Public API

### `new()`
Initializes the contract and sets the caller as the owner.

### `create_proposal(title: String) -> Result<u32, Error>`
Creates a new proposal.  
Only callable by the owner.

### `vote(proposal_id: u32, state: bool) -> Result<(), Error>`
Casts a vote on a proposal.
- `true` → vote in favor
- `false` → vote against

Each account can vote only once per proposal.

### `get_proposal(proposal_id: u32) -> Result<(String, u32, u32), Error>`
Returns the proposal title and current vote counts.

### `total_proposals() -> u32`
Returns the total number of proposals created.

---

## Events

- `ProposalCreated` — emitted when a new proposal is created
- `VoteCast` — emitted when a vote is successfully cast

---

## Error Handling

The contract defines explicit errors for:
- Unauthorized access
- Non-existent proposals
- Double voting attempts
- Arithmetic overflow

This ensures predictable and safe execution.

---

## Testing

The project includes unit tests covering:
- Proposal creation by the owner
- Rejection of proposal creation by non-owners
- Successful voting
- Prevention of double voting
- Handling of invalid proposal IDs

All core logic paths are tested to ensure correctness.

---

## License

This project is licensed under the **MIT License**.


