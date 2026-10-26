/* @layer renderer-app @kind component */
import { Flex, Text, Tooltip } from '@drizztdourden08/tessera';
import type { AgentTimeProps } from '../AgentsTab.type';
import { agentElapsed, tokenLines } from '../model';

const AgentTime = (props: AgentTimeProps) => {
  const { agent, now } = props;
  const lines = tokenLines(agent);
  const elapsed = <Text numeric>{agentElapsed(agent, now)}</Text>;
  if (lines.length === 0) return elapsed;
  return (
    <Tooltip content={<Flex direction="column">{lines.map((line) => <Text key={line} variant="caption">{line}</Text>)}</Flex>}>
      {elapsed}
    </Tooltip>
  );
};

export { AgentTime };
