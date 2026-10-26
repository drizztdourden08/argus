/* @layer renderer-app @kind component */
import { Flex, List, ListItem, Tag, Text } from '@drizztdourden08/tessera';
import type { ThreadListProps } from '../board.type';

const ThreadList = (props: ThreadListProps) => {
  const { threads } = props;
  if (threads.length === 0) return <Text tone="muted">No thread is registered.</Text>;
  return (
    <List aria-label="Threads">
      {threads.map((thread) => (
        <ListItem key={thread.id} bare>
          <Flex gap="sm" align="center">
            <Text>{thread.name}</Text>
            <Tag>{thread.status}</Tag>
          </Flex>
        </ListItem>
      ))}
    </List>
  );
};

export { ThreadList };
