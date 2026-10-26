/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import type { CardFootProps } from '../kanban-props.type';
import { ago } from '../model';
import { CriteriaBar } from './CriteriaBar';

const CardFoot = (props: CardFootProps) => {
  const { card, group, now } = props;
  const { item } = card;
  const showSubject = group !== 'subject' && item.subject && item.subject !== item.thread;
  return (
    <Flex gap="sm" align="center" wrap>
      {showSubject && <Text variant="caption" tone="muted" title={`Subject: ${item.subject}`}>{item.subject}</Text>}
      {group !== 'thread' && <Text variant="caption" tone="muted" title={`Thread: ${item.thread}`}>{item.thread}</Text>}
      <CriteriaBar item={item} />
      <Text variant="caption" tone="muted">{ago(item.updatedAt, now)}</Text>
    </Flex>
  );
};

export { CardFoot };
