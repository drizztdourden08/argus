/* @layer renderer-app @kind component */
import { Callout, Flex, Markdown, Text } from '@drizztdourden08/tessera';
import type { TestBodyProps } from '../tests-props.type';
import { AcList } from './AcList';
import { DrawerSection } from './DrawerSection';
import { Evidence } from './Evidence';
import { ExpectedLists } from './ExpectedLists';
import { OutcomeControls } from './OutcomeControls';
import { RerunButton } from './RerunButton';
import { ResultBlock } from './ResultBlock';
import { ResultHistory } from './ResultHistory';
import { SetupList } from './SetupList';
import { TestFacts } from './TestFacts';
import { TextList } from './TextList';

const TestBody = (props: TestBodyProps) => {
  const { test, item, boardKey, handed } = props;
  return (
    <Flex direction="column" gap="lg">
      <TestFacts test={test} handed={handed} />
      <Flex gap="sm" align="start" wrap>
        <Text variant="label">Run</Text>
        <OutcomeControls test={test} boardKey={boardKey} />
        <RerunButton test={test} boardKey={boardKey} />
      </Flex>
      <ResultBlock test={test} />
      {test.steps.length > 0 && <DrawerSection title="Steps" count={test.steps.length}><TextList ordered lines={test.steps} label="Steps" /></DrawerSection>}
      <ExpectedLists test={test} />
      {test.context && <DrawerSection title="Context"><Markdown size="sm">{test.context}</Markdown></DrawerSection>}
      {test.changed.length > 0 && <DrawerSection title="What changed"><TextList lines={test.changed} label="What changed" /></DrawerSection>}
      <SetupList test={test} />
      <AcList test={test} item={item} />
      <Evidence test={test} boardKey={boardKey} />
      <ResultHistory runs={test.runs} />
      {test.notes.length > 0 && (
        <DrawerSection title="Notes">
          {test.notes.map((note, index) => <Callout key={`${index}-${note}`} tone="info"><Markdown size="sm">{note}</Markdown></Callout>)}
        </DrawerSection>
      )}
    </Flex>
  );
};

export { TestBody };
