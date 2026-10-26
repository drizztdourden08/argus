/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { useMemo } from 'react';
import { useBulkOps } from './behavior/useBulkOps';
import { useDrawer } from './behavior/useDrawer';
import { useFocusState } from './behavior/useFocusState';
import { useItemOps } from './behavior/useItemOps';
import { useKanbanData } from './behavior/useKanbanData';
import { useKanbanState } from './behavior/useKanbanState';
import { useKanbanView } from './behavior/useKanbanView';
import { useNoticeOps } from './behavior/useNoticeOps';
import { useNow } from './behavior/useNow';
import { usePanels } from './behavior/usePanels';
import { useSelection } from './behavior/useSelection';
import type { KanbanTabProps } from './kanban-props.type';
import { subjectsOf } from './model';
import { ItemDrawer } from './sub-components/ItemDrawer';
import { KanbanBody } from './sub-components/KanbanBody';
import { KanbanDialogs } from './sub-components/KanbanDialogs';
import { KanbanToolbar } from './sub-components/KanbanToolbar';
import { OpNotices } from './sub-components/OpNotices';
import { SelectionBar } from './sub-components/SelectionBar';

const KanbanTab = (props: KanbanTabProps) => {
  const { boardKey } = props;
  const data = useKanbanData(boardKey);
  const state = useKanbanState();
  const focus = useFocusState();
  const now = useNow();
  const ops = useItemOps(boardKey);
  const bulk = useBulkOps(boardKey);
  const notices = useNoticeOps(boardKey, data.entries, data.items);
  const { view, choices } = useKanbanView({ data, state, focus, notices, now });
  const drawer = useDrawer(data, view, state.filters);
  const selection = useSelection();
  const panels = usePanels(data, state, focus, bulk);
  const subjects = useMemo(() => subjectsOf(data.items), [data.items]);
  if (!data.ready) return <Text tone="muted">Loading the work items.</Text>;
  return (
    <Flex direction="column" gap="md">
      <KanbanToolbar
        state={state}
        view={view}
        threads={data.threads}
        subjects={subjects}
        focus={focus}
        choices={choices}
        trash={panels.trash}
        onNew={() => panels.adding(null)}
        onPurge={() => panels.purging('thread')}
      />
      <OpNotices ops={notices} />
      {!panels.trash.open && <SelectionBar items={view.cards.map((card) => card.item)} threads={data.threads} selection={selection} ops={bulk} />}
      <KanbanBody
        view={view}
        state={state}
        threads={data.threads}
        now={now}
        selection={selection}
        ops={ops}
        trash={panels.trash}
        onOpen={drawer.open}
        onAdd={panels.adding}
        onPurge={() => panels.purging('trash')}
      />
      <ItemDrawer
        boardKey={boardKey}
        item={drawer.opened}
        tests={data.tests}
        threads={data.threads}
        actions={data.actions}
        now={now}
        ops={ops}
        nav={drawer.nav}
        wide={drawer.wide}
        onWide={drawer.onWide}
        onClose={drawer.close}
      />
      <KanbanDialogs boardKey={boardKey} data={data} focus={focus.focus} ops={ops} dialog={panels.dialog} onClose={panels.close} />
    </Flex>
  );
};

export { KanbanTab };
