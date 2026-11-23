/* @layer renderer-app @kind component */
import { Stack } from '@drizztdourden08/tessera';
import { isActive } from '../model';
import type { ProcessPanelProps } from '../switch.type';
import { ProcessActions } from './ProcessActions';
import { ProcessDetail } from './ProcessDetail';
import { ProcessIdle } from './ProcessIdle';
import { ProcessSummary } from './ProcessSummary';
import './ProcessPanel.css';

const ProcessPanel = (props: ProcessPanelProps) => {
  const { doc, accounts } = props;
  if (!doc || !isActive(doc)) return <ProcessIdle doc={doc} accounts={accounts} />;
  return (
    <Stack gap="sm">
      <ProcessSummary doc={doc} accounts={accounts} />
      <ProcessDetail doc={doc} />
      <ProcessActions doc={doc} />
    </Stack>
  );
};

export { ProcessPanel };
