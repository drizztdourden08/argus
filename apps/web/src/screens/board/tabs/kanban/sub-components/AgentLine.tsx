/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { RecordLink, TypeIcon } from '../../../../../shell';
import type { AgentLineProps } from '../kanban-props.type';
import { AGENT_KIND, AGENT_WORK_TAB } from '../kanban.constants';

const AgentLine = (props: AgentLineProps) => {
  const { item } = props;
  const agents = item.agents ?? [];
  const [first] = agents;
  if (!first) return null;
  const one = agents.length === 1;
  return (
    <Flex gap="xs" align="center" data-agent-line={item.id}>
      <TypeIcon kind={AGENT_KIND} />
      <RecordLink id={one ? first.id : item.id} tab={one ? undefined : AGENT_WORK_TAB}>
        <Text as="span" variant="caption">{one ? first.name : `${agents.length} agents`}</Text>
      </RecordLink>
    </Flex>
  );
};

export { AgentLine };
