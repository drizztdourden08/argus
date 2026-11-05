/* @layer renderer-app @kind component */
import { Button, Flex, Spacer, Text } from '@drizztdourden08/tessera';
import { PASSED_TIP } from '../tests.constants';
import type { ViewRowProps } from '../tests-props.type';

const ViewRow = (props: ViewRowProps) => {
  const { state, view } = props;
  const { prefs, setShowPassed } = state;
  return (
    <Flex gap="md" align="center" wrap>
      <Button size="sm" variant="ghost" title={PASSED_TIP} aria-pressed={prefs.showPassed} onClick={() => setShowPassed(!prefs.showPassed)}>
        {prefs.showPassed ? 'Hide passed' : 'Show passed'}
      </Button>
      <Spacer />
      <Text variant="caption" tone="muted" aria-live="polite">{`${view.toRun} to run / ${view.total}${view.filtered ? ' · filtered' : ''}`}</Text>
    </Flex>
  );
};

export { ViewRow };
