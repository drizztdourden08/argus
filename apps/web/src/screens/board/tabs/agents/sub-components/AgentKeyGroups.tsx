/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { useMemo } from 'react';
import { RecordGroup } from '../../../../../shell';
import { groupAgentsBy } from '../model';
import type { AgentKeyGroupsProps } from '../row-props.type';
import { AgentRow } from './AgentRow';

const AgentKeyGroups = (props: AgentKeyGroupsProps) => {
  const { label, agents, group, ...row } = props;
  const grouped = useMemo(() => (group === 'item' || group === 'none' ? [] : groupAgentsBy(agents, group)), [agents, group]);
  const rows = (members: typeof agents) => members.map((agent) => <AgentRow key={agent.id} id={agent.id} agent={agent} {...row} />);
  if (group === 'none') return <Flex direction="column" gap="xs" aria-label={label}>{rows(agents)}</Flex>;
  return (
    <Flex direction="column" gap="xs" aria-label={label}>
      {grouped.map((entry) => (
        <RecordGroup key={entry.label} header={<Text weight={600}>{entry.label}</Text>} count={entry.rows.length} label={entry.label}>
          {rows(entry.rows)}
        </RecordGroup>
      ))}
    </Flex>
  );
};

export { AgentKeyGroups };
