/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import type { AgentsHeaderProps } from '../AgentsTab.type';
import { LeadLine } from './LeadLine';

const AgentsHeader = (props: AgentsHeaderProps) => {
  const { active, lead, now } = props;
  const count = (status: string): number => active.filter((agent) => agent.status === status).length;
  const stopping = count('stop-requested');
  return (
    <Flex className="agents-head" align="center" wrap>
      <Text as="h2" variant="title" className="agents-head__title">Agents</Text>
      <Text as="span" variant="caption" className="agents-head__count">
        {`working ${count('working')} · waiting ${count('waiting')}${stopping ? ` · stopping ${stopping}` : ''}`}
      </Text>
      {lead && <LeadLine lead={lead} now={now} />}
    </Flex>
  );
};

export { AgentsHeader };
