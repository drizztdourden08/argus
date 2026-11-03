/* @layer renderer-app @kind component */
import { Flex, Tag } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import { useItemOps } from '../tabs/kanban/behavior/useItemOps';
import { KIND_LABEL } from '../tabs/kanban/record';
import { EntryBlock } from '../tabs/kanban/sub-components/EntryBlock';
import { useFeedActions } from '../OwnerFeed';
import { RecordDrawer, useNow } from '../shared';
import { useEntryRow } from './behavior/useEntryRow';
import type { EntryDrawerProps } from './EntryDrawer.type';
import { EntryFacts } from './sub-components/EntryFacts';

const EntryDrawer = (props: EntryDrawerProps) => {
  const { boardKey, id, onClose } = props;
  const row = useEntryRow(boardKey, id);
  const feed = useFeedActions(boardKey);
  const ops = useItemOps(boardKey);
  const now = useNow(id !== null);
  const navigate = useNavigate();
  const entry = row?.entry ?? null;
  const openItem = (itemId: string) => {
    onClose();
    void navigate({ hash: itemId }, { replace: true });
  };
  return (
    <RecordDrawer
      open={entry !== null}
      label="Entry detail"
      wide={false}
      eyebrow={entry && <Tag>{KIND_LABEL[entry.kind] ?? entry.kind}</Tag>}
      title={entry ? entry.id : ''}
      onClose={onClose}
    >
      {entry && row && (
        <Flex direction="column" gap="md">
          <EntryFacts entry={entry} onOpenItem={openItem} />
          <EntryBlock row={row} feed={feed} ops={ops} now={now} />
        </Flex>
      )}
    </RecordDrawer>
  );
};

export { EntryDrawer };
