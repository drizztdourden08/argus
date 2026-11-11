/* @layer renderer-app @kind component */
import { Table, TableCell, TableRow, TableSection, Text } from '@drizztdourden08/tessera';
import { STEP_KIND_LABEL } from '../compare.constants';
import type { StepLogProps } from '../compare.type';
import { recentSteps } from '../model';

const timeOf = (at: string | null | undefined): string => (at ? new Date(at).toLocaleTimeString() : '');

const StepLog = (props: StepLogProps) => {
  const labels = new Map(props.accounts.map((account) => [account.key, account.label ?? account.key]));
  return (
    <Table aria-label="Recent sync steps">
      <TableSection head>
        <TableRow>
          <TableCell header>Time</TableCell>
          <TableCell header>Step</TableCell>
          <TableCell header>Title</TableCell>
          <TableCell header>Account</TableCell>
        </TableRow>
      </TableSection>
      <TableSection>
        {recentSteps(props.steps).map((step) => (
          <TableRow key={step.id}>
            <TableCell><Text variant="caption" smartTimes={false}>{timeOf(step.at)}</Text></TableCell>
            <TableCell>{step.kind ? STEP_KIND_LABEL[step.kind] : ''}</TableCell>
            <TableCell>{step.title ?? step.folder ?? step.detail ?? ''}</TableCell>
            <TableCell>{labels.get(step.account ?? '') ?? step.account ?? ''}</TableCell>
          </TableRow>
        ))}
      </TableSection>
    </Table>
  );
};

export { StepLog };
