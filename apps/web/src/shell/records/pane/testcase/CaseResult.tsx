/* @layer renderer-app @kind component */
import { Callout, Flex, Markdown, Text } from '@drizztdourden08/tessera';
import { useOutcome } from '../../../../screens/board/tabs/tests/behavior/useOutcome';
import { DrawerSection } from '../../../../screens/board/tabs/tests/sub-components/DrawerSection';
import { OutcomeControls } from '../../../../screens/board/tabs/tests/sub-components/OutcomeControls';
import { RerunButton } from '../../../../screens/board/tabs/tests/sub-components/RerunButton';
import { ResultBlock } from '../../../../screens/board/tabs/tests/sub-components/ResultBlock';
import { ResultHistory } from '../../../../screens/board/tabs/tests/sub-components/ResultHistory';
import type { CaseResultProps } from './testcase.type';

const CaseResult = (props: CaseResultProps) => {
  const { test, boardKey } = props;
  const controls = useOutcome(boardKey, test.id);
  const notes = test.notes ?? [];
  return (
    <Flex direction="column" gap="lg">
      <Flex gap="sm" align="center" wrap>
        <Text variant="caption" tone="muted">Run</Text>
        <OutcomeControls test={test} controls={controls} />
        <RerunButton test={test} boardKey={boardKey} />
      </Flex>
      <ResultBlock test={test} />
      <ResultHistory runs={test.runs ?? []} />
      {notes.length > 0 && (
        <DrawerSection title="Notes" count={notes.length}>
          {notes.map((note, index) => <Callout key={`${index}-${note}`} tone="info"><Markdown size="sm">{note}</Markdown></Callout>)}
        </DrawerSection>
      )}
    </Flex>
  );
};

export { CaseResult };
