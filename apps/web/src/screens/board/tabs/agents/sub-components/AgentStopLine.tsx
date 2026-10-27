/* @layer renderer-app @kind component */
import { Button, Text } from '@drizztdourden08/tessera';
import type { AgentFieldProps } from '../drawer-props.type';

const AgentStopLine = (props: AgentFieldProps) => {
  const { id, agent, actions } = props;
  if (agent.status === 'working' || agent.status === 'waiting') {
    return <Button size="xs" variant="danger" onClick={() => actions.stop(id)}>Ask to stop</Button>;
  }
  if (agent.status !== 'stop-requested') return null;
  return (
    <>
      <Button size="xs" variant="secondary" onClick={() => actions.cancelStop(id)}>Cancel the stop request</Button>
      <Text variant="caption">The lead is told at its next check.</Text>
    </>
  );
};

export { AgentStopLine };
