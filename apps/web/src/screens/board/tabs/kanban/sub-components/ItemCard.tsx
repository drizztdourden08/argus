/* @layer renderer-app @kind component */
import { itemCode, RecordCard } from '../../../../../shell';
import type { MouseEvent } from 'react';
import { cardMenuGroups } from '../behavior/card-menu-groups';
import type { ItemCardProps } from '../kanban-props.type';
import { assigneeOf, typeOf } from '../model';
import { CardFoot } from './CardFoot';
import { CardTags } from './CardTags';

const ItemCard = (props: ItemCardProps) => {
  const { boardKey, card, group, threads, now, selected, selecting, ops, onToggle, onMark, onOpen } = props;
  const { item } = card;
  const code = itemCode(item);
  const pick = (event: MouseEvent) => {
    if (selecting || !event.shiftKey) return;
    event.preventDefault();
    event.stopPropagation();
    onMark(true);
    onToggle(item.id);
  };
  return (
    <RecordCard
      head={{
        kind: typeOf(item),
        code,
        assignee: assigneeOf(item),
        boardKey,
        threads,
        menu: cardMenuGroups({ item, threads, ops, onOpen }),
        menuLabel: `More actions for ${code}`,
        selection: { selecting, selected, onToggle: () => onToggle(item.id), onMark },
      }}
      title={item.title}
      openLabel={`Open ${code}: ${item.title}`}
      onOpen={() => onOpen(item.id)}
      signals={<CardTags card={card} now={now} onOpen={onOpen} />}
      foot={<CardFoot card={card} group={group} now={now} />}
      attrs={{ 'data-item': item.id }}
      onClickCapture={pick}
    />
  );
};

export { ItemCard };
