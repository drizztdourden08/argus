/* @layer renderer-app @kind component */
import { Flex, Markdown, Text } from '@drizztdourden08/tessera';
import { EmptyLine } from '../../../../screens/board/tabs/kanban/sub-components/EmptyLine';
import { AcList } from '../../../../screens/board/tabs/tests/sub-components/AcList';
import { DrawerSection } from '../../../../screens/board/tabs/tests/sub-components/DrawerSection';
import { ExpectedLists } from '../../../../screens/board/tabs/tests/sub-components/ExpectedLists';
import { SetupList } from '../../../../screens/board/tabs/tests/sub-components/SetupList';
import { StepList } from '../../../../screens/board/tabs/tests/sub-components/StepList';
import { TestFacts } from '../../../../screens/board/tabs/tests/sub-components/TestFacts';
import { TextList } from '../../../../screens/board/tabs/tests/sub-components/TextList';
import { Age } from '../../../Age';
import type { RecordBodyProps } from '../pane.type';
import { CaseConflicts } from '../testcase/CaseConflicts';
import { useTestCase } from '../testcase/useTestCase';

const TestCaseOverviewTab = (props: RecordBodyProps) => {
  const { data } = props;
  const { test, item, items, handed, ops } = useTestCase(data);
  if (!test) return <EmptyLine>{`Nothing to show for ${data.code}.`}</EmptyLine>;
  return (
    <Flex direction="column" gap="lg">
      <CaseConflicts testId={test.id} boardKey={data.boardKey} />
      <TestFacts test={test} handed={handed} ops={ops} items={items} />
      <Text variant="caption" tone="muted">
        {`rev ${test.rev ?? 1}, updated by ${test.updatedBy ?? test.createdBy ?? 'unknown'} `}<Age value={test.updatedAt} /> ago
      </Text>
      {test.context && <DrawerSection title="Context"><Markdown size="sm">{test.context}</Markdown></DrawerSection>}
      <SetupList test={test} />
      {test.steps.length > 0 && <DrawerSection title="Steps" count={test.steps.length}><StepList lines={test.steps} /></DrawerSection>}
      <ExpectedLists test={test} />
      {test.changed.length > 0 && <DrawerSection title="What changed"><TextList lines={test.changed} label="What changed" /></DrawerSection>}
      <AcList test={test} item={item} />
    </Flex>
  );
};

export { TestCaseOverviewTab };
