/* @layer renderer-app @kind component */
import { Checkbox, Flex, SegmentedControl, Spacer, Text } from '@drizztdourden08/tessera';
import type { ViewRowProps } from '../kanban-props.type';
import { GROUP_OPTIONS, ORDER_OPTIONS, countsText } from '../model';
import type { ItemGroup, ItemOrder } from '../model';

const ViewRow = (props: ViewRowProps) => {
  const { state, view } = props;
  const { prefs, setGroup, setOrder, setShowDone } = state;
  return (
    <Flex gap="md" align="center" wrap>
      <Flex gap="sm" align="center">
        <Text variant="caption" tone="muted">Group by</Text>
        <SegmentedControl<ItemGroup> selectedTone="neutral" aria-label="Group by" size="sm" value={prefs.group} options={GROUP_OPTIONS} onChange={setGroup} />
      </Flex>
      <Flex gap="sm" align="center">
        <Text variant="caption" tone="muted">Order</Text>
        <SegmentedControl<ItemOrder> selectedTone="neutral" aria-label="Order" size="sm" value={prefs.order} options={ORDER_OPTIONS} onChange={setOrder} />
      </Flex>
      <Checkbox size="sm" label={`Show done (${view.closed})`} checked={prefs.showDone} onChange={setShowDone} />
      <Spacer />
      <Text variant="caption" tone="muted" aria-live="polite">{countsText(view)}</Text>
    </Flex>
  );
};

export { ViewRow };
