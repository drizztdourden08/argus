/* @layer renderer-app @kind component */
import { Button, Status, Table, TableCell, TableRow, TableSection, Tag, Text } from '@drizztdourden08/tessera';
import { formatDuration } from '../../../../../format';
import { CHECK_STATUSES } from '../changes.constants';
import type { StepListProps } from '../changes.type';
import { stepStatus } from '../model';

const StepList = (props: StepListProps) => {
  const { run, onLog } = props;
  return (
    <Table aria-label={`Steps of ${run.id}`}>
      <TableSection head>
        <TableRow>
          <TableCell header>Step</TableCell>
          <TableCell header>Status</TableCell>
          <TableCell header align="right">Time</TableCell>
          <TableCell header align="right">Log</TableCell>
        </TableRow>
      </TableSection>
      <TableSection>
        {run.steps.map((step, index) => (
          <TableRow key={`${index}-${step.name}`}>
            <TableCell>
              <Text title={step.command}>{step.name}</Text>
              {step.required === false && <Tag size="md" title="A failure of this step does not block the merge">optional</Tag>}
            </TableCell>
            <TableCell><Status map={CHECK_STATUSES} value={stepStatus(step)} dot size="sm" /></TableCell>
            <TableCell align="right"><Text tone="dim" numeric>{formatDuration(step.durationMs)}</Text></TableCell>
            <TableCell align="right">
              <Button size="sm" variant="ghost" aria-label={`Log of ${run.id} step ${step.name}`} onClick={() => onLog({ runId: run.id, index, name: step.name, command: step.command })}>Log</Button>
            </TableCell>
          </TableRow>
        ))}
      </TableSection>
    </Table>
  );
};

export { StepList };
