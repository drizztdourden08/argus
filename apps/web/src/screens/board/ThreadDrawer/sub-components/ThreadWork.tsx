/* @layer renderer-app @kind component */
import { Button, Flex, Tag, Text } from '@drizztdourden08/tessera';
import { WORK_LIMIT } from '../ThreadDrawer.constants';
import type { ThreadWorkProps } from '../ThreadDrawer.type';
import { ThreadSection } from './ThreadSection';

const ThreadWork = (props: ThreadWorkProps) => {
  const { work, onOpen } = props;
  const hidden = work.length - WORK_LIMIT;
  return (
    <ThreadSection title="Open work" count={work.length}>
      {work.length === 0 && <Text tone="muted">No open work item.</Text>}
      <Flex direction="column" gap="xs" align="start">
        {work.slice(0, WORK_LIMIT).map((item) => (
          <Button key={item.id} size="sm" variant="ghost" aria-label={`Open ${item.id}`} onClick={() => onOpen(item.id)}>
            <Text variant="caption" tone="muted" mono>{item.id}</Text>
            <Tag>{item.status}</Tag>
            <Text>{item.title}</Text>
          </Button>
        ))}
      </Flex>
      {hidden > 0 && <Text tone="muted">{`and ${hidden} more: filter the board by this thread.`}</Text>}
    </ThreadSection>
  );
};

export { ThreadWork };
