/* @layer renderer-app @kind component */
import { SignalStrip } from '@drizztdourden08/tessera';
import { summarySignals } from '../model';
import type { TestsSummaryProps } from '../tests-head-props.type';

const TestsSummary = (props: TestsSummaryProps) => {
  const items = summarySignals(props.view);
  return <SignalStrip items={items} wrap aria-label={items.map((item) => item.label).join(', ')} />;
};

export { TestsSummary };
