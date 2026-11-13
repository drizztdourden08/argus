/* @layer renderer-app @kind component */
import { Flex, Icon, IconButton, Text } from '@drizztdourden08/tessera';
import { COLLAPSE_ALL, EXPAND_ALL } from '../AgentsTab.constants';
import type { AgentsHeaderProps } from '../AgentsTab.type';
import { LeadLine } from './LeadLine';

const AgentsHeader = (props: AgentsHeaderProps) => {
  const { active, lead, now, expand } = props;
  const count = (status: string): number => active.filter((agent) => agent.status === status).length;
  const stopping = count('stop-requested');
  const label = expand.all ? COLLAPSE_ALL : EXPAND_ALL;
  return (
    <Flex className="agents-head" align="center" wrap>
      <Text as="h2" variant="title" className="agents-head__title">Agents</Text>
      <Text as="span" variant="caption" className="agents-head__count">
        {`working ${count('working')} · waiting ${count('waiting')}${stopping ? ` · stopping ${stopping}` : ''}`}
      </Text>
      {lead && <LeadLine lead={lead} now={now} />}
      <IconButton size="sm" variant="ghost" label={label} title={label} aria-label={label} aria-pressed={expand.all} onClick={expand.onToggle}>
        <Icon name={expand.all ? 'chevron-up' : 'chevrons-up-down'} />
      </IconButton>
    </Flex>
  );
};

export { AgentsHeader };
