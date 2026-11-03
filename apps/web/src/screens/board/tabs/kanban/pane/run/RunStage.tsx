/* @layer renderer-app @kind component */
import { Flex, ShortcutList, Text } from '@drizztdourden08/tessera';
import { useRunKeys } from './behavior/useRunKeys';
import { useRunStage } from './behavior/useRunStage';
import { CaseView } from './CaseView';
import { EndRun } from './EndRun';
import { lastResult } from './model/last-result';
import { NoteField } from './NoteField';
import { OutcomeBar } from './OutcomeBar';
import { RunHead } from './RunHead';
import { RUN_KEY_ITEMS } from './run.constants';
import type { RunStageProps } from './run.type';

const RunStage = (props: RunStageProps) => {
  const { data, run, ids, ops } = props;
  const stage = useRunStage(props);
  useRunKeys({ onRecord: stage.record, onStep: stage.step });
  const recorded = lastResult(run, stage.id);
  const left = ids.filter((id) => lastResult(run, id) === null).length;
  return (
    <Flex direction="column" gap="md">
      <RunHead run={run} position={stage.cursor + 1} total={ids.length} steps={stage.steps} onSelect={(id) => stage.goto(ids.indexOf(id))} />
      <CaseView id={stage.id} test={stage.test} open={data.open} />
      {recorded && <Text variant="caption" tone="muted">{`Recorded in this run: ${recorded.status}${recorded.note ? `, ${recorded.note}` : ''}`}</Text>}
      <NoteField value={stage.note} problem={stage.problem} disabled={stage.busy || !stage.test} onChange={stage.setNote} />
      <OutcomeBar disabled={stage.busy || !stage.test} first={stage.cursor === 0} last={stage.cursor === ids.length - 1} onRecord={stage.record} onStep={stage.step} />
      <ShortcutList label="Keys" size="sm" items={RUN_KEY_ITEMS} />
      <EndRun left={left} ops={ops} />
    </Flex>
  );
};

export { RunStage };
