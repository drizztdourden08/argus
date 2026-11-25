/* @layer renderer-app @kind component */
import { Flex, Icon, IconButton, Pressable, Text } from '@drizztdourden08/tessera';
import { useRef } from 'react';
import { agoWords } from '../../../../format';
import { WORDS } from '../boards-words.constants';
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
    <Flex gap="sm" align="end">
      <Flex gap="sm" align="center" wrap className="board-card-foot__meta">
        {card?.updatedAt && <Text variant="caption" tone="muted">{WORDS.updated(agoWords(card.updatedAt, now))}</Text>}
        {listed > 0 && (
          <Pressable onClick={() => pane.open(record.id, 'worktrees')} aria-label={`Open the worktrees of ${record.name}`}>
            <Text variant="caption" tone="muted">{listed} {listed === 1 ? 'worktree' : 'worktrees'}</Text>
          </Pressable>
        )}
        <Text variant="caption" tone={words.tone}>{words.text}</Text>
      </Flex>
      <IconButton ref={anchor} size="sm" variant="ghost" label="Send down" title={`Send a decision or a note to ${record.name}`} onClick={() => send.open(record.key)} className="board-card-foot__send">
        <Icon name="send" />
      </IconButton>
      <SendDownPopover send={send} owner={record.key} anchorRef={anchor} />
    </Flex>
  );
};

export { BoardCardFoot };
