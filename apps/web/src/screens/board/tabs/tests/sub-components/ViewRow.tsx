/* @layer renderer-app @kind component */
import { Flex, SegmentedControl, Text, Toggle } from '@drizztdourden08/tessera';
import { GROUP_OPTIONS, ORDER_OPTIONS } from '../model';
import type { TestGroup, TestOrder } from '../model';
import type { ViewRowProps } from '../tests-props.type';

const ViewRow = (props: ViewRowProps) => {
  const { state, view } = props;
  const { prefs, setGroup, setOrder, setShowPassed } = state;
  return (
    <Flex gap="md" align="center" wrap>
      <SegmentedControl<TestGroup> label="Group by" size="sm" value={prefs.group} options={GROUP_OPTIONS} onChange={setGroup} />
      <SegmentedControl<TestOrder> label="Order" size="sm" value={prefs.order} options={ORDER_OPTIONS} onChange={setOrder} />
      <Toggle size="sm" label="Show passed" checked={prefs.showPassed} onChange={setShowPassed} />
      <Text variant="caption" tone="muted" aria-live="polite">{`${view.toRun} to run / ${view.total}${view.filtered ? ' · filtered' : ''}`}</Text>
    </Flex>
  );
};

export { ViewRow };
