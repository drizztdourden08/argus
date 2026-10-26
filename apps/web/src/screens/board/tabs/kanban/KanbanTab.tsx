/* @layer renderer-app @kind component */
import { EmptyState, Flex, Text } from '@drizztdourden08/tessera';
import { useMemo, useState } from 'react';
import { useBulkOps } from './behavior/useBulkOps';
import { useItemOps } from './behavior/useItemOps';
import { useKanbanData } from './behavior/useKanbanData';
import { useKanbanState } from './behavior/useKanbanState';
import { useNoticeOps } from './behavior/useNoticeOps';
import { useNow } from './behavior/useNow';
import { useSelection } from './behavior/useSelection';
import { NO_ITEMS_TEXT } from './kanban.constants';
import type { KanbanTabProps } from './kanban-props.type';
import { buildView, subjectsOf } from './model';
import { ItemDrawer } from './sub-components/ItemDrawer';
import { KanbanColumns } from './sub-components/KanbanColumns';
import { KanbanToolbar } from './sub-components/KanbanToolbar';
import { AddItemDialog } from './sub-components/AddItemDialog';
import { OpNotices } from './sub-components/OpNotices';
import { SelectionBar } from './sub-components/SelectionBar';

const KanbanTab = (props: KanbanTabProps) => {
  const { boardKey } = props;
  const data = useKanbanData(boardKey);
  const state = useKanbanState();
  const now = useNow();
  const ops = useItemOps(boardKey);
  const bulk = useBulkOps(boardKey);
  const notices = useNoticeOps(boardKey, data.entries, data.items);
  const { items, threads, needs } = data;
  const view = useMemo(
    () => buildView({ items, threads, needs, notices: notices.notices, filters: state.filters, prefs: state.prefs, now }),
    [items, threads, needs, notices.notices, state.filters, state.prefs, now],
  );
  const selection = useSelection();
  const subjects = useMemo(() => subjectsOf(items), [items]);
  const [openId, setOpenId] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  const opened = items.find((item) => item.id === openId && !item.deletedAt) ?? null;
  if (!data.ready) return <Text tone="muted">Loading the work items.</Text>;
  return (
    <Flex direction="column" gap="md">
      <KanbanToolbar state={state} view={view} threads={threads} subjects={subjects} onNew={() => setCreating(true)} />
      <OpNotices ops={notices} />
      <SelectionBar items={view.cards.map((card) => card.item)} threads={threads} selection={selection} ops={bulk} />
      {view.columns.length > 0 ? (
        <KanbanColumns view={view} state={state} threads={threads} now={now} selection={selection} ops={ops} onOpen={setOpenId} />
      ) : (
        <EmptyState message={NO_ITEMS_TEXT} />
      )}
      <ItemDrawer item={opened} tests={data.tests} threads={threads} now={now} ops={ops} onClose={() => setOpenId(null)} />
      <AddItemDialog open={creating} threads={threads} onCreate={(item) => void ops.create(item)} onClose={() => setCreating(false)} />
    </Flex>
  );
};

export { KanbanTab };
