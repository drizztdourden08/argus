/* @layer renderer-app @kind component */
import { DateText, Pressable, Tag, Text, Tooltip } from '@drizztdourden08/tessera';
import { useOpenThread } from '../../shared';
import { chipLook } from '../behavior/chip-look';
import { FAULT_LOOK, PLAIN_LOOK } from '../RegisteredStrip.constants';
import type { RegisteredChipProps } from '../RegisteredStrip.type';
import { AgentText } from './AgentText';
import { ChipMark } from './ChipMark';
import { ChipTip } from './ChipTip';

const RegisteredChip = (props: RegisteredChipProps) => {
  const { row, load } = props;
  const open = useOpenThread();
  const look = chipLook(row);
  const since = row.pendingWake?.since ?? row.since;
  return (
    <Tooltip content={<ChipTip row={row} />}>
      <Pressable aria-label={`Open the thread ${row.thread}`} onClick={() => open(row.thread)}>
        <Tag {...(look.faulted ? FAULT_LOOK : PLAIN_LOOK)} data-state={row.state}>
          <ChipMark look={look} name={row.session ?? row.thread} />
          {look.word ? <Text variant="caption" tone="muted">{look.word}</Text> : null}
          {row.state === 'waking' && since ? <DateText value={since} format="relative" /> : null}
          <AgentText load={load} />
        </Tag>
      </Pressable>
    </Tooltip>
  );
};

export { RegisteredChip };
