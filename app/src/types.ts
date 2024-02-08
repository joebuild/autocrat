import {Autocrat as AutocratIDLType} from '../../target/types/autocrat';
import type {IdlAccounts, IdlTypes} from "@coral-xyz/anchor";

export type UpdateDaoParams = IdlTypes<AutocratIDLType>['UpdateDaoParams'];
export type ProposalInstruction = IdlTypes<AutocratIDLType>['ProposalInstruction'];
