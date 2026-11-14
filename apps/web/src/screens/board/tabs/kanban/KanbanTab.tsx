/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { useMemo } from 'react';
import { useRecordOrder, useRecordPane } from '../../../../shell';
import { useBulkOps } from './behavior/useBulkOps';
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
import { useLayout } from '../kanban-tree';
import { KanbanBody } from './sub-components/KanbanBody';
import { KanbanLegend } from './sub-components/KanbanLegend';
import { KanbanToolbar } from './sub-components/KanbanToolbar';
import { OpNotices } from './sub-components/OpNotices';
import { SelectionBar } from './sub-components/SelectionBar';

const KanbanTab = (props: KanbanTabProps) => {
  const { boardKey, trash, onTrashBack, onPurge } = props;
  const data = useKanbanData(boardKey);
  const state = useKanbanState();
  const focus = useFocusState(boardKey);
  const now = useNow();
  const ops = useItemOps(boardKey);
  const bulk = useBulkOps(boardKey);
  const notices = useNoticeOps(boardKey, data.entries, data.items);
  const { view } = useKanbanView({ data, state, focus, notices, now });
  const pane = useRecordPane();
  const order = useMemo(() => view.columns.flatMap((column) => column.cards.map((card) => card.id)), [view.columns]);
  useRecordOrder(order);
  const selection = useSelection();
  const layout = useLayout();
  const panels = usePanels(data, state, bulk, { open: trash, onBack: onTrashBack });
  if (!data.ready) return <Text tone="muted">Loading the work items.</Text>;
  return (
    <Flex direction="column" gap="md">
      {!panels.trash.open && (
        <KanbanToolbar boardKey={boardKey} state={state} view={view} items={data.items} threads={data.threads} agents={data.agents} selection={selection} layout={layout} />
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
        layout={layout}
        source={{ items: data.items, tests: data.tests, archived: state.filters.archived }}
        onOpen={pane.open}
        onPurge={() => onPurge('trash')}
      />
    </Flex>
  );
};

export { KanbanTab };
