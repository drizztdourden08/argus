/* @layer renderer-app @kind component */
import { CopyValue, Flex, Markdown, Text } from '@drizztdourden08/tessera';
import { DrawerSection } from '../../../../screens/board/tabs/kanban/sub-components/DrawerSection';
import { EmptyLine } from '../../../../screens/board/tabs/kanban/sub-components/EmptyLine';
import { AcList } from '../../../../screens/board/tabs/tests/sub-components/AcList';
import { TestFacts } from '../../../../screens/board/tabs/tests/sub-components/TestFacts';
import { Age } from '../../../Age';
import { FieldLines } from '../fields/FieldLines';
import type { RecordBodyProps } from '../pane.type';
import { CaseConflicts } from '../testcase/CaseConflicts';
import { CaseFields } from '../testcase/CaseFields';
import { useTestCase } from '../testcase/useTestCase';

const TestCaseOverviewTab = (props: RecordBodyProps) => {
  const { data } = props;
  const { test, item, items, ops } = useTestCase(data);
  if (!test) return <EmptyLine>{`Nothing to show for ${data.code}.`}</EmptyLine>;
  const changed = test.changed ?? [];
  return (
    <Flex direction="column" gap="lg">
      <CaseConflicts testId={test.id} boardKey={data.boardKey} />
      <TestFacts test={test} ops={ops} items={items} />
      <Text variant="caption" tone="muted">
        {`rev ${test.rev ?? 1}, updated by ${test.updatedBy ?? test.createdBy ?? 'unknown'} `}<Age value={test.updatedAt} /> ago
      </Text>
      {test.context && <DrawerSection title="Context"><Markdown size="sm">{test.context}</Markdown></DrawerSection>}
      <CaseFields test={test} />
      {test.run && <DrawerSection title="Run command"><CopyValue value={test.run} label="Run command" mono /></DrawerSection>}
      {changed.length > 0 && <DrawerSection title="What changed"><FieldLines label="What changed" lines={changed} /></DrawerSection>}
      <AcList test={test} item={item} />
    </Flex>
  );
};

export { TestCaseOverviewTab };
