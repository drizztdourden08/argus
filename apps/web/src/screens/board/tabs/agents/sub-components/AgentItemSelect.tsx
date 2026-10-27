/* @layer renderer-app @kind component */
import { Flex, Select, Text } from '@drizztdourden08/tessera';
import { useMemo } from 'react';
import type { Item } from '../../../../../data/kanban';
import { useCollection } from '../../../../../data/react';
import type { AgentItemSelectProps } from '../AgentsTab.type';
import { itemOptions } from '../model';

const AgentItemSelect = (props: AgentItemSelectProps) => {
  const { boardKey, id, agent, actions } = props;
  const items = useCollection<Item>(boardKey, 'items');
  const options = useMemo(() => itemOptions(items.docs, agent.item), [items.docs, agent.item]);
  return (
    <Flex gap="xs" align="center">
      <Text as="span" variant="caption">Item</Text>
      <Select
        aria-label="Item"
        size="sm"
        options={options}
        value={agent.item ?? ''}
        onChange={(item) => item !== (agent.item ?? '') && actions.setItem(id, item === '' ? null : item)}
      />
    </Flex>
  );
};

export { AgentItemSelect };
