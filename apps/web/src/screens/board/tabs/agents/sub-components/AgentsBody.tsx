/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { EMPTY_MATCH, FINISHED_TITLE, WORKING_TITLE } from '../AgentsTab.constants';
import type { AgentsBodyProps } from '../AgentsTab.type';
import { EMPTY_ALL, EMPTY_RUNNING } from '../model';
import { AgentSection } from './AgentSection';

const AgentsBody = (props: AgentsBodyProps) => {
  const { boardKey, view, showFinished, group, layout, now, actions, onOpen, open, onRowOpen } = props;
  const { active, done, items, threads, shown } = view;
  if (active.length + done.length === 0) return <Text as="p" tone="muted" className="agents-empty">{EMPTY_ALL}</Text>;
  const shared = { boardKey, items, threads, group, layout, now, actions, onOpen, open, onRowOpen };
  return (
    <Flex direction="column" gap="md" className="agents-body">
      {shown.active.length === 0
        ? <Text as="p" tone="muted" className="agents-empty">{active.length === 0 ? EMPTY_RUNNING : EMPTY_MATCH}</Text>
        : <AgentSection title={WORKING_TITLE} label="Working agents" agents={shown.active} {...shared} />}
      {showFinished && shown.done.length > 0 && <AgentSection title={FINISHED_TITLE} label="Finished agents" agents={shown.done} {...shared} />}
    </Flex>
  );
};

export { AgentsBody };
