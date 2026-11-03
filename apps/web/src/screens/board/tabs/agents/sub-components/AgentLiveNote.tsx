/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { DrawerSection } from '../../kanban/sub-components/DrawerSection';
import type { AgentLiveNoteProps } from '../row-props.type';
import { agentNow } from '../model';

const AgentLiveNote = (props: AgentLiveNoteProps) => {
  const { agent } = props;
  const doing = agentNow(agent);
  return (
    <DrawerSection title="Now">
      <Text tone={doing ? undefined : 'muted'}>{doing?.text ?? agent.live?.liveNote ?? 'Nothing running right now.'}</Text>
    </DrawerSection>
  );
};

export { AgentLiveNote };
