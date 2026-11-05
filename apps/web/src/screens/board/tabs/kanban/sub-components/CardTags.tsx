/* @layer renderer-app @kind component */
import { SignalStrip } from '@drizztdourden08/tessera';
import { useOwnerProfile } from '../../../../../owner';
import { itemCode, useRecordPane } from '../../../../../shell';
import type { CardTagsProps } from '../kanban-props.type';
import { signalItems, stripItem } from '../signals';

const CardTags = (props: CardTagsProps) => {
  const { card, now, onOpen } = props;
  const pane = useRecordPane();
  const owner = useOwnerProfile();
  const opener = { item: () => onOpen(itemCode(card.item)), entry: pane.open };
  return <SignalStrip items={signalItems(card, now).map((signal) => stripItem(signal, opener, owner))} iconOnly wrap aria-label="Signals" />;
};

export { CardTags };
