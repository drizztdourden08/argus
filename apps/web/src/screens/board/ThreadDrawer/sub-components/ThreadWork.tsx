/* @layer renderer-app @kind component */
import { Box, Pressable, Tag, Text } from '@drizztdourden08/tessera';
import { WORK_LIMIT } from '../ThreadDrawer.constants';
import type { ThreadWorkProps } from '../ThreadDrawer.type';
import { ThreadSection } from './ThreadSection';

const ThreadWork = (props: ThreadWorkProps) => {
  const { work, onOpen } = props;
  const hidden = work.length - WORK_LIMIT;
  return (
    <ThreadSection title="Open work" count={work.length}>
      {work.length === 0 && <Text tone="muted">No open work item.</Text>}
      <Box className="thread-work">
        {work.slice(0, WORK_LIMIT).map((item) => (
          <Pressable key={item.id} className="thread-work__item" aria-label={`Open ${item.id}`} onClick={() => onOpen(item.id)}>
            <Text as="span" variant="caption" mono weight={600} className="thread-work__id">{item.id}</Text>
            <Tag>{item.status}</Tag>
            <Text as="span" className="thread-work__title">{item.title}</Text>
          </Pressable>
        ))}
      </Box>
      {hidden > 0 && <Text variant="caption">{`and ${hidden} more: filter the board by this thread.`}</Text>}
    </ThreadSection>
  );
};

export { ThreadWork };
