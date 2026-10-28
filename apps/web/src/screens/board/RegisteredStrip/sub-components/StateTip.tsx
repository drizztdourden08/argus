/* @layer renderer-app @kind component */
import { Age } from '../../../../shell';
import { whoOf } from '../behavior/who-of';
import { BUSY_TIP, DEAF_NOTE } from '../RegisteredStrip.constants';
import type { ChipTipProps } from '../RegisteredStrip.type';

const StateTip = (props: ChipTipProps) => {
  const { row } = props;
  const who = whoOf(row);
  if (row.state === 'waking') {
    return (
      <>
        {who}: waking up, a wake was sent <Age value={row.pendingWake?.since ?? row.since ?? ''} ago /> and has not
        been answered yet
      </>
    );
  }
  if (row.state === 'busy') return <>{who}: {BUSY_TIP}</>;
  if (row.state === 'listening') return <>{who}: listening{row.listenerLabel ? ` (${row.listenerLabel})` : ''}</>;
  return <>{who}: not listening, so the dashboard cannot wake it{row.deaf ? DEAF_NOTE : ''}</>;
};

export { StateTip };
