/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import type { EntrySectionProps } from '../drawer-props.type';
import { DrawerSection } from './DrawerSection';
import { EmptyLine } from './EmptyLine';
import { EntryBlock } from './EntryBlock';

const EntrySection = (props: EntrySectionProps) => {
  const { title, empty, rows, feed, ops, now } = props;
  return (
    <DrawerSection title={title}>
      {rows.length === 0 && <EmptyLine>{empty}</EmptyLine>}
      <Flex direction="column" gap="md">
        {rows.map((row) => (
          <EntryBlock key={row.entry.id} row={row} feed={feed} ops={ops} now={now} />
        ))}
      </Flex>
    </DrawerSection>
  );
};

export { EntrySection };
