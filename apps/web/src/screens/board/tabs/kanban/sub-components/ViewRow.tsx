/* @layer renderer-app @kind component */
import { Button, Flex, SegmentedControl, Spacer, Text } from '@drizztdourden08/tessera';
import { DONE_TIP } from '../kanban.constants';
import type { ViewRowProps } from '../kanban-props.type';
import { countsText } from '../model';
import { LAYOUT_OPTIONS } from '../../kanban-tree';
import type { KanbanLayout } from '../../kanban-tree';

const ViewRow = (props: ViewRowProps) => {
  const { state, view, selection, layout } = props;
  const { prefs, setShowDone } = state;
  return (
    <Flex gap="md" align="center" wrap>
      <Flex gap="sm" align="center">
        <Text variant="caption" tone="muted">View</Text>
        <SegmentedControl<KanbanLayout> selectedTone="neutral" aria-label="View" size="sm-plus" value={layout.layout} options={LAYOUT_OPTIONS} onChange={layout.setLayout} />
      </Flex>
      <Button size="sm" variant="ghost" title={DONE_TIP} aria-pressed={prefs.showDone} onClick={() => setShowDone(!prefs.showDone)}>
        {`${prefs.showDone ? 'Hide' : 'Show'} done (${view.closed})`}
      </Button>
      <Button size="sm" variant="ghost" aria-pressed={selection.active} onClick={() => selection.setActive(!selection.active)}>Select</Button>
      <Spacer />
      <Text variant="caption" tone="muted" aria-live="polite">{countsText(view)}</Text>
    </Flex>
  );
};

export { ViewRow };
