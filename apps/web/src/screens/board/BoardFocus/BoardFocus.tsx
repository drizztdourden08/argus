/* @layer renderer-app @kind component */
import { Button, Flex, Select, Tooltip } from '@drizztdourden08/tessera';
import { useFocusChoices } from './behavior/useFocusChoices';
import { ALL_THREADS, CLEAR_LABEL } from './BoardFocus.constants';
import type { BoardFocusProps } from './BoardFocus.type';

const BoardFocus = (props: BoardFocusProps) => {
  const { boardKey } = props;
  const { focus, showPaused, choices } = useFocusChoices(boardKey);
  const focused = choices.options.find((option) => option.value === focus.focus);
  const select = (
    <Select
      size="sm"
      aria-label="Focus"
      prefix="Focus:"
      options={[{ value: ALL_THREADS, label: 'All threads' }, ...choices.options]}
      value={focused ? focused.value : ALL_THREADS}
      onChange={(value) => focus.setFocus(value === ALL_THREADS ? '' : value)}
    />
  );
  return (
    <Flex gap="sm" align="center" inline>
      {focused ? <Tooltip content={`Only ${focused.label} is shown. Its open questions are in Activity.`}>{select}</Tooltip> : select}
      {focused && <Button size="sm" variant="ghost" onClick={() => focus.setFocus('')}>{CLEAR_LABEL}</Button>}
      {choices.pausedCount > 0 && (
        <Button size="sm" variant="ghost" active={showPaused.on} aria-pressed={showPaused.on} onClick={() => showPaused.set(!showPaused.on)}>
          {`Show paused (${choices.pausedCount})`}
        </Button>
      )}
    </Flex>
  );
};

export { BoardFocus };
