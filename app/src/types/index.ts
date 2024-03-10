import { Autocrat as AutocratIDLType } from './autocrat';

import type { IdlAccounts, IdlTypes } from "@coral-xyz/anchor";

export type UpdateDaoParams = IdlTypes<AutocratIDLType>['UpdateDaoParams'];
export type ProposalInstruction = IdlTypes<AutocratIDLType>['ProposalInstruction'];

export type Proposal = IdlAccounts<AutocratIDLType>['proposal'];
export type Dao = IdlAccounts<AutocratIDLType>['dao'];
