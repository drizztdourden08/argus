/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { BoardMarkdown } from '../../../../../shell';
import type { AgentDrawerBodyProps } from '../drawer-props.type';
import { isActive } from '../model';
import { AgentDrawerHead } from './AgentDrawerHead';
import { AgentDrawerTools } from './AgentDrawerTools';
import { AgentTextField } from './AgentTextField';
import { AgentTimeline } from './AgentTimeline';
import { AgentWhere } from './AgentWhere';
import { DrawerSection } from './DrawerSection';

const AgentDrawerBody = (props: AgentDrawerBodyProps) => {
  const { boardKey, id, agent, order, now, actions, wide, onWide, onOpen } = props;
  const noResult = isActive(agent) ? 'Not finished yet.' : 'No result recorded.';
  return (
    <Flex direction="column" gap="lg">
      <AgentDrawerTools id={id} agent={agent} order={order} actions={actions} wide={wide} onWide={onWide} onOpen={onOpen} />
      <AgentDrawerHead boardKey={boardKey} id={id} agent={agent} now={now} actions={actions} />
      <DrawerSection title="Task"><AgentTextField id={id} agent={agent} actions={actions} field="task" /></DrawerSection>
      <DrawerSection title="Note"><AgentTextField id={id} agent={agent} actions={actions} field="note" /></DrawerSection>
      <DrawerSection title="Result">
        {agent.result ? <BoardMarkdown>{agent.result}</BoardMarkdown> : <Text tone="muted">{noResult}</Text>}
      </DrawerSection>
      <DrawerSection title="Where"><AgentWhere id={id} agent={agent} actions={actions} /></DrawerSection>
      <DrawerSection title="Timeline"><AgentTimeline agent={agent} now={now} /></DrawerSection>
    </Flex>
  );
};

export { AgentDrawerBody };
