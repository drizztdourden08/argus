/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { DrawerSection } from '../../../../screens/board/tabs/kanban/sub-components/DrawerSection';
import { EmptyLine } from '../../../../screens/board/tabs/kanban/sub-components/EmptyLine';
import { TimelineLine } from '../../../../screens/board/tabs/kanban/sub-components/TimelineLine';
import type { RecordBodyProps } from '../pane.type';

const RecordActivityTab = (props: RecordBodyProps) => {
  const { code, record, acts, now, doc } = props.data;
  const thread = typeof doc?.thread === 'string' ? doc.thread : '';
  return (
    <DrawerSection title={`Activity · ${record.timeline.length}`}>
      {record.timeline.length === 0 && <EmptyLine>{`Nothing yet on ${code}.`}</EmptyLine>}
      <Flex direction="column" gap="xs" role="list" aria-label="Activity">
        {record.timeline.map((row) => (
          <TimelineLine key={row.key} row={row} thread={thread} acts={acts} now={now} />
        ))}
      </Flex>
    </DrawerSection>
  );
};

export { RecordActivityTab };
