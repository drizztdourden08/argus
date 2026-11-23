/* @layer renderer-app @kind component */
import { Inline, Tag } from '@drizztdourden08/tessera';
import { SIGNAL_URGENCY } from '../boards.constants';
import type { BoardSignalsProps } from '../boards.type';

const BoardSignals = (props: BoardSignalsProps) => {
  const { signals, onOpen } = props;
  if (signals.length === 0) return null;
  return (
    <Inline gap="xs" wrap aria-label="Signals">
      {signals.map((signal) => {
        const color = SIGNAL_URGENCY[signal.tone];
        const open = () => onOpen(signal.tab);
        return color
          ? <Tag key={signal.id} variant="urgency" color={color} title={signal.label} onClick={open}>{signal.text}</Tag>
          : <Tag key={signal.id} title={signal.label} onClick={open}>{signal.text}</Tag>;
      })}
    </Inline>
  );
};

export { BoardSignals };
