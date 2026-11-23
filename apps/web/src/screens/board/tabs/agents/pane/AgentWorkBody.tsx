/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import type { RecordBodyProps } from '../../../../../shell/records/pane/pane.type';
import { DrawerSection } from '../../kanban/sub-components/DrawerSection';
import { EmptyLine } from '../../kanban/sub-components/EmptyLine';
import { AgentItemsField } from '../sub-components/AgentItemsField';
import { BuildRunRow } from '../../builds/sub-components/BuildRunRow';
import { AgentStopLine } from '../sub-components/AgentStopLine';
import { AgentWhere } from '../sub-components/AgentWhere';
import { RecordLink } from '../../../../../shell';
import { useAgentActions } from '../behavior/useAgentActions';
import { useAgentDoc } from '../behavior/useAgentDoc';
import { useAgentRuns } from '../behavior/useAgentRuns';
import { agentItems, NO_BLOCKERS_LINE, NO_RUNS_LINE } from '../model';

const AgentWorkBody = (props: RecordBodyProps) => {
  const { boardKey, target, now } = props.data;
  const { agent, agents, ready } = useAgentDoc(boardKey, target.id);
  const actions = useAgentActions(boardKey, agents);
  const runs = useAgentRuns(boardKey);
  if (!agent) return <Text tone="muted">{ready ? `${target.id} is not on this board.` : 'Loading the agent.'}</Text>;
  const mine = runs.all.filter((run) => run.agent === agent.id);
  const blockers = agent.blockedBy ?? [];
  return (
    <Flex direction="column" gap="lg">
      <Flex gap="sm" align="center" wrap>
        <AgentStopLine id={agent.id} agent={agent} actions={actions} />
      </Flex>
      <DrawerSection title={`Items · ${agentItems(agent).length}`}>
        <AgentItemsField boardKey={boardKey} id={agent.id} agent={agent} actions={actions} />
      </DrawerSection>
      <DrawerSection title="Where it works">
        <AgentWhere id={agent.id} agent={agent} actions={actions} />
      </DrawerSection>
      <DrawerSection title={`Blocked by · ${blockers.length}`}>
        {blockers.length === 0 && <EmptyLine>{NO_BLOCKERS_LINE}</EmptyLine>}
        <Flex gap="sm" wrap role="list" aria-label="Blocked by">
          {blockers.map((blocker) => <RecordLink key={blocker} id={blocker} board={boardKey} />)}
        </Flex>
      </DrawerSection>
      <DrawerSection title={`Runs · ${mine.length}`}>
        {mine.length === 0 && <EmptyLine>{NO_RUNS_LINE}</EmptyLine>}
        <Flex direction="column" gap="xs" role="list" aria-label="Runs of the agent">
          {mine.map((run) => <BuildRunRow key={run.id} run={run} boardKey={boardKey} now={now} />)}
        </Flex>
      </DrawerSection>
    </Flex>
  );
};

export { AgentWorkBody };
