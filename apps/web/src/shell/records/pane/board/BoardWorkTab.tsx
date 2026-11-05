/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { RecordLink } from '../../sub-components/RecordLink';
import { BoardList } from './BoardList';
import { BoardRow } from './BoardRow';
import type { BoardTabProps } from './board.type';
import { liveAgents } from './model/live-agents';

const BoardWorkTab = (props: BoardTabProps) => {
  const { data } = props;
  const agents = liveAgents(data.agents);
  const blocked = data.items.filter((item) => !item.deletedAt && !item.archivedAt && ((item.blocker ?? '') !== '' || (item.blocked?.length ?? 0) > 0));
  return (
    <Flex direction="column" gap="lg">
      <BoardList title="Live agents" empty="No agent is working." count={agents.length}>
        {agents.map((agent) => (
          <BoardRow key={agent.id}>
            <Text as="span" weight={600}>{agent.name ?? agent.id}</Text>
            <Text as="span" tone="muted">{agent.task ?? agent.status}</Text>
          </BoardRow>
        ))}
      </BoardList>
      <BoardList title="Blocked" empty="Nothing is blocked." count={blocked.length}>
        {blocked.map((item) => (
          <BoardRow key={item.id}>
            <RecordLink id={item.id} board={data.record.key} />
            <Text as="span">{item.title}</Text>
          </BoardRow>
        ))}
      </BoardList>
    </Flex>
  );
};

export { BoardWorkTab };
