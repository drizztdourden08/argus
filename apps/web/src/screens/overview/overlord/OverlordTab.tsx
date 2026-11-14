/* @layer renderer-app @kind component */
import { Box, Callout, EmptyState } from '@drizztdourden08/tessera';
import { useChatOpen } from '../../../shell/OverlordChat';
import { useOverlord } from './behavior/useOverlord';
import { READING } from './overlord.constants';
import { ApprovalStrip } from './sub-components/ApprovalStrip';
import { BudgetCard } from './sub-components/BudgetCard';
import { ModeCard } from './sub-components/ModeCard';
import { SpendCard } from './sub-components/SpendCard';
import { StatusBar } from './sub-components/StatusBar';
import './OverlordTab.css';

const OverlordTab = () => {
  const overlord = useOverlord();
  const chat = useChatOpen();
  const { status, spend } = overlord;
  if (!status) return overlord.problem ? <Callout tone="danger">{`The Overlord could not be read: ${overlord.problem}.`}</Callout> : <EmptyState message={READING} />;
  return (
    <Box className="overlord-layout">
      <Box className="overlord-layout__status">
        <StatusBar status={status} action={overlord.control} onControl={overlord.onControl} chatOpen={chat.open} onChat={chat.toggle} />
      </Box>
      <Box className="overlord-layout__approvals">
        <ApprovalStrip approvals={overlord.pending} action={overlord.decision} onDecide={overlord.onDecide} />
      </Box>
      <Box className="overlord-layout__mode"><ModeCard mode={status.mode} action={overlord.modeAction} onSwitch={overlord.onSwitch} /></Box>
      {spend && <Box className="overlord-layout__spend"><SpendCard spend={spend} /></Box>}
      {spend && <Box className="overlord-layout__budget"><BudgetCard spend={spend} action={overlord.budget} onSave={overlord.onSave} /></Box>}
    </Box>
  );
};

export { OverlordTab };
