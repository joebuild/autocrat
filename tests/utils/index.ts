import { assert } from "chai";
import { Clock, ProgramTestContext } from "solana-bankrun";

export const fastForward = async (
  context: ProgramTestContext,
  slots: bigint
) => {
  const currentClock = await context.banksClient.getClock();
  context.setClock(
    new Clock(
      currentClock.slot + slots,
      currentClock.epochStartTimestamp,
      currentClock.epoch,
      currentClock.leaderScheduleEpoch,
      50n
    )
  );
};

export const expectFailure = async (action: Promise<any>) => {
  try {
    await action;
    assert(false);
  } catch (err) {}
};
