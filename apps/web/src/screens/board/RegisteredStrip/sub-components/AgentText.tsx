/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import type { AgentTextProps } from '../RegisteredStrip.type';

const AgentText = (props: AgentTextProps) => {
  const { load } = props;
  return (
    <>
      {load.working > 0 ? (
        <Text variant="caption" tone="muted">+{load.working} {load.working > 1 ? 'agents' : 'agent'} working</Text>
      ) : null}
      {load.stalled > 0 ? <Text variant="caption" tone="warning">{load.stalled} stalled</Text> : null}
    </>
  );
};

export { AgentText };
