/* @layer renderer-app @kind component */
import { Flex, Icon, IconButton, Pressable, Spacer, Text } from '@drizztdourden08/tessera';
import { useRef } from 'react';
import { formatAgo } from '../../../../format';
import { useRecordPane } from '../../../../shell/records';
import { PORTFOLIO_SCOPE, useDoc } from '../../../../data/react';
import type { PortfolioWorktrees } from '../../../../data/worktrees';
import { SendDownPopover } from './SendDownPopover';
import { syncWords } from '../behavior/sync-words';
import type { BoardCardFootProps } from '../boards.type';

const BoardCardFoot = (props: BoardCardFootProps) => {
  const { record, card, now, send, sync } = props;
  const pane = useRecordPane();
  const anchor = useRef<HTMLButtonElement>(null);
  const listed = useDoc<PortfolioWorktrees>(PORTFOLIO_SCOPE, 'meta', 'worktrees').doc?.counts?.[record.key]?.listed ?? 0;
  const words = syncWords(sync, now);
  return (
    <Flex gap="sm" align="center" wrap className="board-card__foot">
      {card?.updatedAt && <Text variant="caption" tone="muted">Updated {formatAgo(card.updatedAt, now)} ago</Text>}
      {listed > 0 && (
        <Pressable onClick={() => pane.open(record.id, 'worktrees')} aria-label={`Open the worktrees of ${record.name}`}>
          <Text variant="caption" tone="muted">{listed} {listed === 1 ? 'worktree' : 'worktrees'}</Text>
        </Pressable>
      )}
      <Text variant="caption" tone={words.tone}>{words.text}</Text>
      <Spacer />
      <IconButton ref={anchor} size="sm" variant="ghost" label="Send down" title={`Send a decision or a note to ${record.name}`} onClick={() => send.open(record.key)}>
        <Icon name="send" />
      </IconButton>
      <SendDownPopover send={send} owner={record.key} anchorRef={anchor} />
    </Flex>
  );
};

export { BoardCardFoot };
