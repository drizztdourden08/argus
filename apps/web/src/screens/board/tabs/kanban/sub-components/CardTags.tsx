/* @layer renderer-app @kind component */
import { SignalStrip } from '@drizztdourden08/tessera';
import { useRecordPane } from '../../../../../shell';
import type { CardTagsProps } from '../kanban-props.type';
import { CARD_SIGNAL_MAX, signalItems, stripItem } from '../signals';

const CardTags = (props: CardTagsProps) => {
  const { card, now, onOpen } = props;
  const pane = useRecordPane();
  const opener = { item: () => onOpen(card.id), entry: pane.open };
  return <SignalStrip items={signalItems(card, now).map((signal) => stripItem(signal, opener))} max={CARD_SIGNAL_MAX} aria-label="Signals" />;
};

export { CardTags };
