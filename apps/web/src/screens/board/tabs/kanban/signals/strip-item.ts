/* @layer renderer-app @kind logic */
import type { SignalItem } from '@drizztdourden08/tessera';
import type { CardSignal, SignalOpener } from './signal.type';

const stripItem = (signal: CardSignal, opener: SignalOpener): SignalItem => {
  const { id, icon, label, tone, opens } = signal;
  const onClick = () => (opens.kind === 'entry' ? opener.entry(opens.id) : opener.item());
  return signal.count === undefined ? { id, icon, label, tone, onClick } : { id, icon, label, tone, count: signal.count, onClick };
};

export { stripItem };
