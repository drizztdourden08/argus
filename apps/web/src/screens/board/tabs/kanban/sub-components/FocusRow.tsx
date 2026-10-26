/* @layer renderer-app @kind component */
import { Button, Flex, Select, Text, Toggle } from '@drizztdourden08/tessera';
import { ALL_THREADS } from '../kanban.constants';
import type { FocusRowProps } from '../kanban-props.type';

const FocusRow = (props: FocusRowProps) => {
  const { focus, choices } = props;
  const focused = choices.options.find((option) => option.value === focus.focus);
  return (
    <Flex gap="sm" align="center" wrap>
      <Select
        aria-label="Focus"
        size="sm"
        options={[{ value: ALL_THREADS, label: 'All threads' }, ...choices.options]}
        value={focused ? focused.value : ALL_THREADS}
        onChange={(value) => focus.setFocus(value === ALL_THREADS ? '' : value)}
      />
      {focused && <Button size="sm" variant="ghost" onClick={() => focus.setFocus('')}>Show all</Button>}
      {focused && <Text variant="caption" tone="muted">{`Only ${focused.label} is shown. Its open questions are in Activity.`}</Text>}
      {choices.pausedCount > 0 && (
        <Toggle size="sm" label={`Show paused (${choices.pausedCount})`} checked={focus.showPaused} onChange={focus.setShowPaused} />
      )}
    </Flex>
  );
};

export { FocusRow };
