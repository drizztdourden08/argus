/* @layer renderer-app @kind logic */
import { recordMenuGroups } from '../../../../../shell';
import type { RecordHeadProps } from '../../../../../shell';
import type { AgentCardProps } from '../row-props.type';
import { agentHandlers } from './agent-handlers';

const agentHead = (props: Pick<AgentCardProps, 'boardKey' | 'id' | 'agent' | 'threads' | 'actions' | 'onOpen'>): RecordHeadProps => {
  const { boardKey, id, agent, threads, actions, onOpen } = props;
  return {
    kind: 'agent',
    code: id,
    assignee: agent.thread ?? '',
    boardKey,
    threads,
    menu: recordMenuGroups('agent', id, agentHandlers(id, agent, actions, onOpen)),
    menuLabel: `More actions for ${id}`,
  };
};

export { agentHead };
