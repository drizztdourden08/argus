/* @layer renderer-app @kind component */
import { Flex, Icon, IconButton, Spacer, Text } from '@drizztdourden08/tessera';
import { useRef } from 'react';
import { formatAgo } from '../../../../format';
import { SyncStamp, WorktreeCounter } from '../../../../shell';
import { PORTFOLIO_SCOPE, useDoc } from '../../../../data/react';
import type { PortfolioWorktrees } from '../../../../data/worktrees';
import { CardProgressRow } from '../../../board/tabs/kanban/sub-components/CardProgressRow';
import { SendDownPopover } from '../../dashboards/sub-components/SendDownPopover';
import { agentsText } from '../behavior/agents-text';
import { progressSegments } from '../behavior/progress-segments';
import type { BoardCardFootProps } from '../boards.type';
import { boardProgress } from '../model';
import { BoardPlay } from './BoardPlay';

const BoardCardFoot = (props: BoardCardFootProps) => {
  const { record, card, now, send, run, sync, onLog } = props;
  const anchor = useRef<HTMLButtonElement>(null);
  const counter = useDoc<PortfolioWorktrees>(PORTFOLIO_SCOPE, 'meta', 'worktrees').doc?.counts?.[record.key];
  return (
    <Flex direction="column" gap="xs">
      <Flex gap="sm" align="center" wrap>
        <Text variant="caption" tone="muted" title="Agents">{agentsText(record.counts)}</Text>
        <Text variant="caption" tone="muted" title="Lead state">{record.lead}</Text>
        <CardProgressRow segments={progressSegments(boardProgress(record))} />
        <Spacer />
        <Text variant="caption" tone="muted" title="Board updated">{formatAgo(card?.updatedAt, now)}</Text>
      </Flex>
      {record.started && (
        <Flex gap="sm" align="center" wrap>
          {card && <BoardPlay card={card} run={run} onLog={onLog} />}
          {counter && <WorktreeCounter listed={counter.listed} running={counter.running} />}
          <SyncStamp sync={sync} compact />
          <Spacer />
          <IconButton ref={anchor} size="sm" variant="ghost" label="Send down" title={`Send a decision or a note to ${record.name}`} onClick={() => send.open(record.key)}>
            <Icon name="send" />
          </IconButton>
          <SendDownPopover send={send} owner={record.key} anchorRef={anchor} />
        </Flex>
      )}
    </Flex>
  );
};

export { BoardCardFoot };
