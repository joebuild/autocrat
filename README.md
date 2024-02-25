# The MetaDAO v2 Programs and SDK

# Table of Contents
1. [Programs](#programs)
    * [AMM](#amm)
    * [Autocrat](#autocrat)
2. [SDK](#sdk)

## Programs

### AMM
The AMM uses the simple constant product curve model `k=x*y`.

Importantly, it has an added time-weighted average price (TWAP). More specifically it also incorporates liquidity-weighting into the calculations ([see here](https://github.com/joebuild/autocrat/blob/master/programs/amm/src/state/amm.rs#L59)). This type of TWAP in an AMM will no longer require "cranking", as it is updated on every: swap, liquidity deposit, and liquidity withdrawal.

The AMM also has permissioned functionality, which means that each pool can be configured to only allow calls from a specific program/caller (in this case the `Autocrat`).

The AMM is called via CPI from `Autocrat`, and has checks surrounding the lifetime of the proposal cycle (for example, it prevents swaps after the proposal is finalized, so that people don't have to worry about immediately withdrawing to prevent being arbed as the spot price changes).

### Autocrat
This program has two main functions (which could be broken out into two separate programs in the future, to facilitate other use cases). The first is the DAO, which manages proposals (and running transactions on passing proposals). The second is the conditional vault, which is responsible for minting and redeeming conditional tokens.

This version of the Autocrat supports multiple instructions to be run after a proposal has passed. These instrucitons can be uploaded one at a time to get around the standard stack limit restrictions.

The process for submitting a proposal is as follows:
1. `create_proposal`
2. `create_proposal_instructions`
3. `add_proposal_instructions` (optional additional instructions)
4. `create_proposal_market_side` (start with either pass or fail)
5. `create_proposal_market_side` (the other market)
6. `submit_proposal`

When `create_proposal` is called the proposer has to deposit the comitted META and USDC up front.

The architecture of `create_proposal_market_side` was inteded to be easily modified to enable multi-choice proposals in the future. Each time this is called, an AMM market is created, and the proposer is minted the conditional tokens (corresponding to their LP deposit). 

The proposer is responsible for setting the initial conditional market prices. If the proposer estimates poorly, then they will lose money after price corrections from savvy analysts.

This program also includes the 'merge conditional tokens' functionality, i.e. 1 pMETA + 1 fMETA can been redeemed at any time for 1 META (and similarly for USDC).

## SDK

This repository has an SDK for each program listed above, to enable simple frontend integration and access for programmatic traders. Additionally, there is some helpful tooling for adding priority fees and changing the requested compute units (see below).

```
let ixh = await autocratClient.submitProposal(
    proposalKeypair,
    proposalInstructionsAddress,
    "https://proposals.com/10"
);
await ixh
    .setComputeUnits(400_000)
    .setPriorityFee(100)
    .rpc(opts);
```