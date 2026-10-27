/* @layer renderer-app @kind component */
import { Button, Flex, Select, Text } from '@drizztdourden08/tessera';
import { ALL_THREADS } from '../tests.constants';
import type { FocusRowProps } from '../tests-props.type';

const FocusRow = (props: FocusRowProps) => {
  const { focus } = props;
  const { scope, options, pausedCount, setFocus, setShowPaused } = focus;
  const focused = options.find((option) => option.value === scope.focus);
  return (
    <Flex gap="sm" align="center" wrap>
      <Text variant="caption" tone="muted" aria-hidden="true">Focus</Text>
      <Select
        size="sm"
        className="tests-fit"
        aria-label="Focus"
        options={[{ value: ALL_THREADS, label: 'All threads' }, ...options.map((option) => ({ value: option.value, label: option.paused ? `${option.label} (paused)` : option.label }))]}
        value={focused ? focused.value : ALL_THREADS}
        onChange={(value) => setFocus(value === ALL_THREADS ? '' : value)}
      />
      {focused && <Button size="sm" variant="ghost" onClick={() => setFocus('')}>Show all</Button>}
      {focused && <Text variant="caption" tone="muted">{`Only ${focused.label} is shown. Its open questions are in Activity.`}</Text>}
      {pausedCount > 0 && (
        <Button size="sm" variant="ghost" aria-pressed={scope.showPaused} onClick={() => setShowPaused(!scope.showPaused)}>
          {`${scope.showPaused ? 'Hide' : 'Show'} paused (${pausedCount})`}
        </Button>
      )}
    </Flex>
  );
};

export { FocusRow };
