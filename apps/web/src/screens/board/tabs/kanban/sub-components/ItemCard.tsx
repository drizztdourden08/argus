/* @layer renderer-app @kind component */
import { Checkbox, Flex, Pressable, Spacer, Text } from '@drizztdourden08/tessera';
import type { MouseEvent } from 'react';
import { HEAD_ID_CLASS, HEAD_ROW_CLASS } from '../head.constants';
import type { ItemCardProps } from '../kanban-props.type';
import { assigneeOf } from '../model';
import { AssigneeMark } from './AssigneeMark';
import { CardFoot } from './CardFoot';
import { CardMenu } from './CardMenu';
import { CardTags } from './CardTags';
import { ItemTypeMark } from './ItemTypeMark';
import './HeadRow.css';

const ItemCard = (props: ItemCardProps) => {
  const { boardKey, card, group, threads, now, selected, selecting, ops, onToggle, onMark, onOpen } = props;
  const { item } = card;
  const pick = (event: MouseEvent) => {
    if (selecting || !event.shiftKey) return;
    event.preventDefault();
    event.stopPropagation();
    onMark(true);
    onToggle(item.id);
  };
  return (
    <Flex direction="column" gap="xs" data-item={item.id} onClickCapture={pick}>
      <Flex gap="xs" align="center" className={HEAD_ROW_CLASS}>
        {selecting ? (
          <Flex onClickCapture={(event) => onMark(event.shiftKey)}>
            <Checkbox checked={selected} onChange={() => onToggle(item.id)} ariaLabel={`Select ${item.id}`} size="sm" />
          </Flex>
        ) : (
          <ItemTypeMark item={item} />
        )}
        <Text variant="caption" tone="muted" mono className={HEAD_ID_CLASS}>{item.id}</Text>
        <Spacer />
        <AssigneeMark assignee={assigneeOf(item)} boardKey={boardKey} threads={threads} />
        <CardMenu item={item} threads={threads} ops={ops} onOpen={onOpen} />
      </Flex>
      <Pressable onClick={() => onOpen(item.id)} aria-label={`Open ${item.id}: ${item.title}`}>
        <Text weight={500} clamp={3} smartTimes>{item.title}</Text>
      </Pressable>
      <CardTags card={card} now={now} onOpen={onOpen} />
      <CardFoot card={card} group={group} now={now} />
    </Flex>
  );
};

export { ItemCard };
