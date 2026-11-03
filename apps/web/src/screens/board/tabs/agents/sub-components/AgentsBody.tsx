/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { FINISHED_TITLE, WORKING_TITLE } from '../AgentsTab.constants';
import type { AgentsBodyProps } from '../AgentsTab.type';
import { EMPTY_ALL, EMPTY_RUNNING } from '../model';
import { AgentSection } from './AgentSection';

const AgentsBody = (props: AgentsBodyProps) => {
  const { boardKey, view, showFinished, now, actions, onOpen } = props;
  const { active, done, items, threads } = view;
  if (active.length + done.length === 0) return <Text as="p" tone="muted" className="agents-empty">{EMPTY_ALL}</Text>;
  const shared = { boardKey, items, threads, now, actions, onOpen };
  return (
    <Flex direction="column" gap="md" className="agents-body">
      {active.length === 0
        ? <Text as="p" tone="muted" className="agents-empty">{EMPTY_RUNNING}</Text>
        : <AgentSection title={WORKING_TITLE} label="Working agents" agents={active} {...shared} />}
      {showFinished && done.length > 0 && <AgentSection title={FINISHED_TITLE} label="Finished agents" agents={done} {...shared} />}
    </Flex>
  );
};

export { AgentsBody };
