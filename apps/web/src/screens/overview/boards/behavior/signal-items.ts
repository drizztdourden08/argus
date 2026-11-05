/* @layer renderer-app @kind logic */
import type { SignalItem } from '@drizztdourden08/tessera';
import { SIGNAL_ICON, SIGNAL_TONE } from '../boards.constants';
import type { BoardSignal } from '../model';

const signalItems = (signals: readonly BoardSignal[], open: (tab: string) => void): SignalItem[] =>
  signals.map((signal) => ({
    id: signal.id,
    icon: SIGNAL_ICON[signal.id],
    label: signal.label,
    tone: SIGNAL_TONE[signal.tone],
    ...(signal.count > 1 ? { count: signal.count } : {}),
    onClick: () => open(signal.tab),
  }));

export { signalItems };
