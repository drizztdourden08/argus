/* @layer renderer-app @kind component */
import { Button, Flex, SegmentedControl, Spacer, Text } from '@drizztdourden08/tessera';
import { GROUP_OPTIONS, ORDER_OPTIONS } from '../model';
import type { TestGroup, TestOrder } from '../model';
import { PASSED_TIP } from '../tests.constants';
import type { ViewRowProps } from '../tests-props.type';

const ViewRow = (props: ViewRowProps) => {
  const { state, view } = props;
  const { prefs, setGroup, setOrder, setShowPassed } = state;
  return (
    <Flex gap="md" align="center" wrap>
      <Flex gap="sm" align="center">
        <Text variant="caption" tone="muted" aria-hidden="true">Group by</Text>
        <SegmentedControl<TestGroup> selectedTone="neutral" aria-label="Group by" size="sm-plus" value={prefs.group} options={GROUP_OPTIONS} onChange={setGroup} />
      </Flex>
      <Flex gap="sm" align="center">
        <Text variant="caption" tone="muted" aria-hidden="true">Order</Text>
        <SegmentedControl<TestOrder> selectedTone="neutral" aria-label="Order" size="sm-plus" value={prefs.order} options={ORDER_OPTIONS} onChange={setOrder} />
      </Flex>
      <Button size="sm" variant="ghost" title={PASSED_TIP} aria-pressed={prefs.showPassed} onClick={() => setShowPassed(!prefs.showPassed)}>
        {prefs.showPassed ? 'Hide passed' : 'Show passed'}
      </Button>
      <Spacer />
      <Text variant="caption" tone="muted" aria-live="polite">{`${view.toRun} to run / ${view.total}${view.filtered ? ' · filtered' : ''}`}</Text>
    </Flex>
  );
};

export { ViewRow };
