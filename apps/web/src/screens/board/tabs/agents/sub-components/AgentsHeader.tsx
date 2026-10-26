/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import type { AgentsHeaderProps } from '../AgentsTab.type';
import { LeadLine } from './LeadLine';

const AgentsHeader = (props: AgentsHeaderProps) => {
  const { active, lead } = props;
  const count = (status: string): number => active.filter((agent) => agent.status === status).length;
  const stopping = count('stop-requested');
  return (
    <Flex justify="between" align="center" wrap gap="md">
      <Text tone="muted">
        {`working ${count('working')}, waiting ${count('waiting')}${stopping ? `, stopping ${stopping}` : ''}`}
      </Text>
      {lead && <LeadLine lead={lead} />}
    </Flex>
  );
};

export { AgentsHeader };
