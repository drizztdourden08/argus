/* @layer renderer-app @kind component */
import { Button, Flex, SegmentedControl, Text, Toggle } from '@drizztdourden08/tessera';
import type { ViewRowProps } from '../kanban-props.type';
import { GROUP_OPTIONS, ORDER_OPTIONS, countsText } from '../model';
import type { ItemGroup, ItemOrder } from '../model';

const ViewRow = (props: ViewRowProps) => {
  const { state, view, onNew } = props;
  const { prefs, setGroup, setOrder, setShowDone } = state;
  return (
    <Flex gap="md" align="center" wrap>
      <SegmentedControl<ItemGroup> label="Group by" size="sm" value={prefs.group} options={GROUP_OPTIONS} onChange={setGroup} />
      <SegmentedControl<ItemOrder> label="Order" size="sm" value={prefs.order} options={ORDER_OPTIONS} onChange={setOrder} />
      <Toggle size="sm" label={`Show done (${view.closed})`} checked={prefs.showDone} onChange={setShowDone} />
      <Text variant="caption" tone="muted" aria-live="polite">{countsText(view)}</Text>
      <Button size="sm" variant="primary" onClick={onNew}>New item</Button>
    </Flex>
  );
};

export { ViewRow };
