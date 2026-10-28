/* @layer renderer-app @kind component */
import './TestRow.css';
import { Box, Pressable, Text } from '@drizztdourden08/tessera';
import { smartTimes } from '../../../../../format';
import { rowKeys } from '../behavior/row-keys';
import { useOutcome } from '../behavior/useOutcome';
import { useRowMenu } from '../behavior/useRowMenu';
import { useTestsShared } from '../behavior/useTestsShared';
import { isToRun, resultNote } from '../model';
import type { TestPartProps } from '../tests-props.type';
import { DoneSummary } from './DoneSummary';
import { OutcomeButtons } from './OutcomeButtons';
import { ResultComposer } from './ResultComposer';
import { TestChips } from './TestChips';
import { TestMenu } from './TestMenu';

const rowClass = (status: string, toRun: boolean, composing: boolean, conflict: boolean): string =>
  ['test-row', toRun ? 'test-row--run' : `test-row--done test-row--${status}`, composing && 'test-row--composing', conflict && 'test-row--conflict']
    .filter(Boolean)
    .join(' ');

const TestRow = (props: TestPartProps) => {
  const { test } = props;
  const { boardKey, onOpen, conflicted } = useTestsShared();
  const controls = useOutcome(boardKey, test.id);
  const menu = useRowMenu();
  const toRun = isToRun(test);
  const note = toRun ? '' : resultNote(test);
  return (
    <Box
      className={rowClass(test.status, toRun, controls.open !== null, conflicted.has(test.id))}
      data-test={test.id}
      data-open-test={test.id}
      data-to-run={toRun ? '' : undefined}
      onKeyDown={rowKeys(toRun, controls, menu)}
    >
      <Text mono variant="caption" tone="muted" className="test-row__id">{test.id}</Text>
      <Box className="test-row__body">
        <Pressable className="test-row__title" title={test.title} onClick={() => onOpen(test.id)}>
          <Text weight={500}>{smartTimes(test.title)}</Text>
        </Pressable>
        {note && <Text variant="caption" tone="muted" title={note} className="test-row__note">{note}</Text>}
        <TestChips test={test} />
      </Box>
      <Box className="test-row__result">
        {toRun ? <OutcomeButtons testId={test.id} active={controls.open} busy={controls.busy} onChoose={controls.choose} /> : <DoneSummary test={test} />}
      </Box>
      <Box className="test-row__menu"><TestMenu test={test} menu={menu} /></Box>
      {controls.open && (
        <Box className="test-row__composer"><ResultComposer testId={test.id} outcome={controls.open} controls={controls} /></Box>
      )}
    </Box>
  );
};

export { TestRow };
