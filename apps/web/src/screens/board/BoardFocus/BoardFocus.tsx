/* @layer renderer-app @kind component */
import { Button, Flex, Select } from '@drizztdourden08/tessera';
import { useFocusChoices } from './behavior/useFocusChoices';
import { ALL_THREADS } from './BoardFocus.constants';
import type { BoardFocusProps } from './BoardFocus.type';

const BoardFocus = (props: BoardFocusProps) => {
  const { boardKey } = props;
  const { focus, showPaused, choices } = useFocusChoices(boardKey);
  const focused = choices.options.find((option) => option.value === focus.focus);
  return (
    <Flex gap="sm" align="center" wrap inline>
      <Select
        size="sm"
        aria-label="Focus"
        prefix="Focus:"
        options={[{ value: ALL_THREADS, label: 'All threads' }, ...choices.options]}
        value={focused ? focused.value : ALL_THREADS}
        onChange={(value) => focus.setFocus(value === ALL_THREADS ? '' : value)}
      />
      {choices.pausedCount > 0 && (
        <Button size="sm" variant="ghost" onClick={() => showPaused.set(!showPaused.on)}>
          {`${showPaused.on ? 'Hide' : 'Show'} paused (${choices.pausedCount})`}
        </Button>
      )}
    </Flex>
  );
};

export { BoardFocus };
