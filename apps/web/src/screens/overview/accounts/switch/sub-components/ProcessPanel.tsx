/* @layer renderer-app @kind component */
import { Card, Stack } from '@drizztdourden08/tessera';
import { isActive } from '../model';
import { PROCESS_TITLE } from '../switch.constants';
import type { ProcessPanelProps } from '../switch.type';
import { ProcessActions } from './ProcessActions';
import { ProcessDetail } from './ProcessDetail';
import { ProcessIdle } from './ProcessIdle';
import { ProcessSummary } from './ProcessSummary';
import './ProcessPanel.css';

const ProcessPanel = (props: ProcessPanelProps) => {
  const { doc, accounts } = props;
  if (!doc || !isActive(doc)) {
    return <Card title={PROCESS_TITLE}><ProcessIdle doc={doc} accounts={accounts} /></Card>;
  }
  return (
    <Card title={PROCESS_TITLE} actions={<ProcessActions doc={doc} />}>
      <Stack gap="sm">
        <ProcessSummary doc={doc} accounts={accounts} />
        <ProcessDetail doc={doc} />
      </Stack>
    </Card>
  );
};

export { ProcessPanel };
