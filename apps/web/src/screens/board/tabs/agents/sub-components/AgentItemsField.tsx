/* @layer renderer-app @kind component */
import { Flex, Icon, IconButton, Select, Text } from '@drizztdourden08/tessera';
import { useCollection } from '../../../../../data/react';
import type { Item } from '../../../../../data/kanban';
import { RecordLink } from '../../../../../shell';
import type { AgentItemSelectProps } from '../drawer-props.type';
import { agentItems, itemOptions, NO_ITEM, NO_ITEM_LINE } from '../model';
import { EmptyLine } from '../../kanban/sub-components/EmptyLine';

const AgentItemsField = (props: AgentItemSelectProps) => {
  const { boardKey, id, agent, actions } = props;
  const items = useCollection<Item>(boardKey, 'items');
  const linked = agentItems(agent);
  const titleOf = (item: string): string => items.docs.find((doc) => doc.id === item)?.title ?? '';
  return (
    <Flex direction="column" gap="xs">
      {linked.length === 0 && <EmptyLine>{NO_ITEM_LINE}</EmptyLine>}
      <Flex direction="column" gap="2xs" role="list" aria-label="Items of the agent">
        {linked.map((item) => (
          <Flex key={item} gap="xs" align="center" role="listitem">
            <RecordLink id={item} board={boardKey} />
            <Text clamp={1}>{titleOf(item)}</Text>
            <IconButton label={`Remove ${item} from ${id}`} variant="ghost" size="2xs" onClick={() => actions.setItems(id, linked.filter((other) => other !== item))}>
              <Icon name="x" />
            </IconButton>
          </Flex>
        ))}
      </Flex>
      <Select
        aria-label="Add an item"
        size="sm"
        options={itemOptions(items.docs, linked)}
        value={NO_ITEM}
        onChange={(item) => item !== NO_ITEM && actions.setItems(id, [...linked, item])}
      />
    </Flex>
  );
};

export { AgentItemsField };
