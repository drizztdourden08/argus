/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import type { EntrySectionProps } from '../drawer-props.type';
import { DrawerSection } from './DrawerSection';
import { EntryBlock } from './EntryBlock';

const EntrySection = (props: EntrySectionProps) => {
  const { title, empty, rows, feed, ops, now } = props;
  return (
    <DrawerSection title={title}>
      {rows.length === 0 && <Text tone="muted">{empty}</Text>}
      <Flex direction="column" gap="md">
        {rows.map((row) => (
          <EntryBlock key={row.entry.id} row={row} feed={feed} ops={ops} now={now} />
        ))}
      </Flex>
    </DrawerSection>
  );
};

export { EntrySection };
