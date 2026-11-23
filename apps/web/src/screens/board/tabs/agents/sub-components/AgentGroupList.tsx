/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { useMemo } from 'react';
import { RecordGroup } from '../../../../../shell';
import { CLOSED_ITEMS_TITLE, groupAgents, NO_ITEM_TITLE } from '../model';
import type { AgentGroupListProps } from '../row-props.type';
import { AgentCells } from './AgentCells';
import { ItemGroup } from './ItemGroup';

const AgentGroupList = (props: AgentGroupListProps) => {
  const { label, agents, items, ...row } = props;
  const grouped = useMemo(() => groupAgents(agents, items), [agents, items]);
  const group = (key: string, members: typeof agents) => <ItemGroup key={key} itemId={key} items={items} agents={members} {...row} />;
  return (
    <Flex direction="column" gap="xs" aria-label={label}>
      {grouped.groups.map((entry) => group(entry.key, entry.rows))}
      {grouped.none.length > 0 && (
        <RecordGroup header={<Text weight={600}>{NO_ITEM_TITLE}</Text>} count={grouped.none.length} flagged label={NO_ITEM_TITLE}>
          <AgentCells agents={grouped.none} {...row} />
        </RecordGroup>
      )}
      {grouped.closed.length > 0 && (
        <RecordGroup header={<Text weight={600}>{CLOSED_ITEMS_TITLE}</Text>} count={grouped.closed.length} defaultOpen={false} label={CLOSED_ITEMS_TITLE}>
          {grouped.closed.map((entry) => group(entry.key, entry.rows))}
        </RecordGroup>
      )}
    </Flex>
  );
};

export { AgentGroupList };
