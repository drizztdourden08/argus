/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { BoardMarkdown } from '../../../../shell';
import type { ThreadBodyProps } from '../ThreadDrawer.type';
import { ThreadAgents } from './ThreadAgents';
import { ThreadBrief } from './ThreadBrief';
import { ThreadFacts } from './ThreadFacts';
import { ThreadSection } from './ThreadSection';
import { ThreadWork } from './ThreadWork';

const ThreadBody = (props: ThreadBodyProps) => {
  const { view, now, ops, onOpenItem } = props;
  const { thread, work } = view;
  const agents = thread.agents ?? [];
  return (
    <Flex direction="column" gap="lg">
      <ThreadFacts thread={thread} now={now} ops={ops} />
      <ThreadSection title="Now">{thread.note ? <BoardMarkdown>{thread.note}</BoardMarkdown> : <Text tone="muted">No note.</Text>}</ThreadSection>
      {agents.length > 0 && <ThreadAgents agents={agents} now={now} />}
      {thread.brief && <ThreadBrief thread={thread} ops={ops} />}
      <ThreadWork work={work} onOpen={onOpenItem} />
    </Flex>
  );
};

export { ThreadBody };
