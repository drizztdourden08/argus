/* @layer renderer-app @kind component */
import { Status } from '@drizztdourden08/tessera';
import { statusTip, STATUS_TONE } from '../model';
import type { StatusCellProps } from '../processes.type';

const StatusCell = (props: StatusCellProps) => {
  const { proc } = props;
  const status = proc.status ?? '';
  return <Status variant="pill" tone={STATUS_TONE[status] ?? 'neutral'} title={statusTip(proc)}>{status}</Status>;
};

export { StatusCell };
