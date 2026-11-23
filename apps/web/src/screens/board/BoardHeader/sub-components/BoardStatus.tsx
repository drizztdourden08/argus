/* @layer renderer-app @kind component */
import { FactsPanel, Icon, Status } from '@drizztdourden08/tessera';
import type { FactsPanelFact } from '@drizztdourden08/tessera';
import { HeaderPopover } from '../../../../shell/HeaderPopover';
import { RegisteredStrip } from '../../RegisteredStrip';
import type { BoardStatusProps } from '../BoardHeader.type';
import { BoardFaults } from './BoardFaults';
import { BoardRunning } from './BoardRunning';
import { BoardStamp } from './BoardStamp';
import { BoardSync } from './BoardSync';
import { BoardWorktrees } from './BoardWorktrees';
import { LeadStatus } from './LeadStatus';

const BoardStatus = (props: BoardStatusProps) => {
  const { boardKey, health } = props;
  const { tone, reason, lead, faults, listed, runningAgents, status, updatedAt } = health;
  const counted = typeof listed === 'number' && typeof runningAgents === 'number';
  const facts: FactsPanelFact[] = [
    { label: 'Lead', value: <LeadStatus lead={lead} /> },
    { label: 'Updated', value: <BoardStamp status={status} updatedAt={updatedAt} /> },
    ...(counted ? [
      { label: 'Worktrees', value: <BoardWorktrees listed={listed} runningAgents={runningAgents} /> },
      { label: 'Running agents', value: <BoardRunning runningAgents={runningAgents} /> },
    ] : []),
    { label: 'Last sync', value: <BoardSync boardKey={boardKey} /> },
    { label: 'Registered threads', value: <RegisteredStrip boardKey={boardKey} /> },
    { label: 'Faults', value: faults.length > 0 ? <BoardFaults faults={faults} /> : 'none' },
  ];
  return (
    <HeaderPopover trigger={<><Status tone={tone} dot>{null}</Status><Icon name="info" size={14} aria-label="Board status" /></>} label="Board status" hint={reason}>
      <FactsPanel layout="terms" framed={false} label="Board status" groups={[facts]} />
    </HeaderPopover>
  );
};

export { BoardStatus };
