/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import type { ActivitySectionProps } from '../drawer-props.type';
import { Composer } from './Composer';
import { DrawerSection } from './DrawerSection';
import { EmptyLine } from './EmptyLine';
import { TimelineLine } from './TimelineLine';

const ActivitySection = (props: ActivitySectionProps) => {
  const { item, rows, acts, now } = props;
  return (
    <DrawerSection title={`Activity · ${rows.length}`}>
      <Composer item={item} acts={acts} />
      {rows.length === 0 && <EmptyLine>Nothing yet on this item.</EmptyLine>}
      <Flex direction="column" gap="xs" role="list" aria-label="Activity">
        {rows.map((row) => (
          <TimelineLine key={row.key} row={row} thread={item.thread} acts={acts} now={now} />
        ))}
      </Flex>
    </DrawerSection>
  );
};

export { ActivitySection };
