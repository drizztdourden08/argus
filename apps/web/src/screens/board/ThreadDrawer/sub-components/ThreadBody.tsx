/* @layer renderer-app @kind component */
import { Flex, Markdown, Text } from '@drizztdourden08/tessera';
import type { ThreadBodyProps } from '../ThreadDrawer.type';
import { ThreadAgents } from './ThreadAgents';
import { ThreadBrief } from './ThreadBrief';
import { ThreadFacts } from './ThreadFacts';
import { ThreadSection } from './ThreadSection';
import { ThreadTools } from './ThreadTools';
import { ThreadWork } from './ThreadWork';

const ThreadBody = (props: ThreadBodyProps) => {
  const { view, ops, wide, onWide, onPurge, onOpenItem } = props;
  const { thread, counts, work, paused } = view;
  const agents = thread.agents ?? [];
  return (
    <Flex direction="column" gap="lg">
      <ThreadTools thread={thread} paused={paused} ops={ops} wide={wide} onWide={onWide} onPurge={onPurge} />
      <ThreadFacts thread={thread} counts={counts} ops={ops} />
      <ThreadSection title="Now">{thread.note ? <Markdown>{thread.note}</Markdown> : <Text tone="muted">No note.</Text>}</ThreadSection>
      {agents.length > 0 && <ThreadAgents agents={agents} />}
      {thread.brief && <ThreadBrief thread={thread} ops={ops} />}
      <ThreadWork work={work} onOpen={onOpenItem} />
    </Flex>
  );
};

export { ThreadBody };
