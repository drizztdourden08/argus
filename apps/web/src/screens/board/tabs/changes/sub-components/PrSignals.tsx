/* @layer renderer-app @kind component */
import { SignalStrip } from '@drizztdourden08/tessera';
import { useRecordPane } from '../../../../../shell';
import { PR_PANE_TAB, SIGNAL_MAX } from '../changes.constants';
import type { PrSignalsProps } from '../changes.type';
import { prSignals } from '../model';

const PrSignals = (props: PrSignalsProps) => {
  const { pr, runs } = props;
  const pane = useRecordPane();
  const signals = prSignals(pr, runs);
  if (signals.length === 0) return null;
  const items = signals.map((signal) => ({ ...signal, onClick: () => pane.open(pr.id, PR_PANE_TAB) }));
  return <SignalStrip items={items} max={SIGNAL_MAX} aria-label="Signals" />;
};

export { PrSignals };
