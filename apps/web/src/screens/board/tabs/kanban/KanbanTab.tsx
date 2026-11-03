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
import { itemTests, needsYou } from './model';
import { ItemDrawer } from './sub-components/ItemDrawer';
import { KanbanBody } from './sub-components/KanbanBody';
import { KanbanLegend } from './sub-components/KanbanLegend';
import { KanbanToolbar } from './sub-components/KanbanToolbar';
import { OpNotices } from './sub-components/OpNotices';
import { SelectionBar } from './sub-components/SelectionBar';

const KanbanTab = (props: KanbanTabProps) => {
  const { boardKey, trash, onTrashBack, onPurge } = props;
  const data = useKanbanData(boardKey);
  const state = useKanbanState();
  const focus = useFocusState();
  const now = useNow();
  const ops = useItemOps(boardKey);
  const bulk = useBulkOps(boardKey);
  const notices = useNoticeOps(boardKey, data.entries, data.items);
  const { view } = useKanbanView({ data, state, focus, notices, now });
  const drawer = useDrawer(data, view, state.filters);
  const selection = useSelection();
  const panels = usePanels(data, state, bulk, { open: trash, onBack: onTrashBack });
  const drawerTests = useMemo(() => itemTests(data.tests, data.closedBuilds, state.filters.archived), [data.tests, data.closedBuilds, state.filters.archived]);
  const blocking = (drawer.opened ? data.asks.get(drawer.opened.id) : undefined)?.find((link) => link.via === 'blocker') ?? null;
  if (!data.ready) return <Text tone="muted">Loading the work items.</Text>;
  return (
    <Flex direction="column" gap="md">
      {!panels.trash.open && (
        <KanbanToolbar boardKey={boardKey} state={state} view={view} items={data.items} threads={data.threads} selection={selection} />
      )}
      {!panels.trash.open && <KanbanLegend />}
      <OpNotices ops={notices} />
      {!panels.trash.open && <SelectionBar items={view.cards.map((card) => card.item)} threads={data.threads} selection={selection} ops={bulk} />}
      <KanbanBody
        boardKey={boardKey}
        view={view}
        state={state}
        threads={data.threads}
        now={now}
        selection={selection}
        ops={ops}
        trash={panels.trash}
        add={panels.add}
        focus={focus.focus}
        onOpen={drawer.open}
        onPurge={() => onPurge('trash')}
      />
      <ItemDrawer
        boardKey={boardKey}
        item={drawer.opened}
        items={data.items}
        tests={drawerTests}
        threads={data.threads}
        actions={data.actions}
        waiting={needsYou(drawer.opened ? data.needs.get(drawer.opened.id) : undefined)}
        blocking={blocking}
        now={now}
        ops={ops}
        nav={drawer.nav}
        wide={drawer.wide}
        onWide={drawer.onWide}
        onClose={drawer.close}
      />
    </Flex>
  );
};

export { KanbanTab };
