/* @layer renderer-app @kind component */
import { SignalStrip } from '@drizztdourden08/tessera';
import { useRecordPane } from '../../../../../shell';
import { SIGNAL_MAX } from '../builds.constants';
import type { BuildSignalsProps } from '../builds-props.type';
import { buildSignals } from '../model';

const BuildSignals = (props: BuildSignalsProps) => {
  const { view, code } = props;
  const pane = useRecordPane();
  const signals = buildSignals(view);
  if (signals.length === 0) return null;
  const items = signals.map(({ id, icon, label, tone, tab }) => ({ id, icon, label, tone, onClick: () => pane.open(code, tab) }));
  return <SignalStrip items={items} max={SIGNAL_MAX} aria-label="Signals" />;
};

export { BuildSignals };
