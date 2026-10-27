/* @layer renderer-app @kind component */
import { Box, Text } from '@drizztdourden08/tessera';
import type { AgentTimeProps } from '../row-props.type';
import { agentElapsed, tokenLines } from '../model';
import { AgentCell } from './AgentCell';

const countOf = (value: number | undefined): string => Number(value ?? 0).toLocaleString('en-US');

const timeTip = (props: AgentTimeProps): string => {
  const { agent } = props;
  const started = agent.startedAt ? `Started ${new Date(agent.startedAt).toLocaleString()}` : '';
  const lines = tokenLines(agent);
  return lines.length ? `${started}\n${lines.join('\n')}` : started;
};

const AgentTime = (props: AgentTimeProps) => {
  const { agent, now } = props;
  return (
    <AgentCell name="time" title={timeTip(props)}>
      <Box as="span">{agentElapsed(agent, now)}</Box>
      {agent.live && <Text as="span" variant="caption">{`${countOf(agent.live.turns)} · ${countOf(agent.live.toolCalls)}`}</Text>}
    </AgentCell>
  );
};

export { AgentTime };
