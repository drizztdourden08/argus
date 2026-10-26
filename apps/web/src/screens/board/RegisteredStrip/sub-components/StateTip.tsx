/* @layer renderer-app @kind component */
import { DateText } from '@drizztdourden08/tessera';
import { whoOf } from '../behavior/who-of';
import { BUSY_TIP, DEAF_NOTE } from '../RegisteredStrip.constants';
import type { ChipTipProps } from '../RegisteredStrip.type';

const StateTip = (props: ChipTipProps) => {
  const { row } = props;
  const who = whoOf(row);
  if (row.state === 'waking') {
    return (
      <>
        {who}: waking up, a wake was sent <DateText value={row.pendingWake?.since ?? row.since ?? ''} format="relative" /> and has not
        been answered yet
      </>
    );
  }
  if (row.state === 'busy') return <>{who}: {BUSY_TIP}</>;
  if (row.state === 'listening') return <>{who}: listening{row.listenerLabel ? ` (${row.listenerLabel})` : ''}</>;
  return <>{who}: not listening, so the dashboard cannot wake it{row.deaf ? DEAF_NOTE : ''}</>;
};

export { StateTip };
