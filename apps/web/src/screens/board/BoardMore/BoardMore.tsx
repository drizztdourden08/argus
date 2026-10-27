/* @layer renderer-app @kind component */
import { DropdownMenu } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { useStoredFlag } from '../shared';
import { useMoreCounts } from './behavior/useMoreCounts';
import { HIDDEN_KEY, HIDDEN_TIP } from './BoardMore.constants';
import type { BoardMoreProps, MoreDialog } from './BoardMore.type';
import { MoreDialogs } from './sub-components/MoreDialogs';

const BoardMore = (props: BoardMoreProps) => {
  const { boardKey, trashCount, onTrash, onPurge } = props;
  const counts = useMoreCounts(boardKey);
  const [dialog, setDialog] = useState<MoreDialog>(null);
  const [showHidden, setShowHidden] = useStoredFlag(HIDDEN_KEY);
  return (
    <>
      <DropdownMenu
        label="More"
        trigger={{ label: 'More', icon: 'ellipsis', iconOnly: true }}
        groups={[
          {
            id: 'board',
            items: [
              { id: 'trash', label: `Trash (${trashCount})`, onSelect: onTrash },
              { id: 'locks', label: `Resources and locks (${counts.held} held)`, onSelect: () => setDialog('locks') },
              { id: 'legend', label: 'Legend', onSelect: () => setDialog('legend') },
              { id: 'hidden', label: `${showHidden ? 'Hide' : 'Show'} hidden (${counts.hidden})`, description: HIDDEN_TIP, onSelect: () => setShowHidden(!showHidden) },
              { id: 'purge', label: 'Purge threads and records...', tone: 'danger', onSelect: onPurge },
            ],
          },
        ]}
        variant="ghost"
        size="sm"
        align="end"
      />
      <MoreDialogs dialog={dialog} locks={counts.locks} held={counts.held} onClose={() => setDialog(null)} />
    </>
  );
};

export { BoardMore };
