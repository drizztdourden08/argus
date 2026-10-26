/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import { STATUS_LOOK, statusTip, UNKNOWN_STATUS_LOOK } from '../model';
import type { StatusCellProps } from '../processes.type';

const StatusCell = (props: StatusCellProps) => {
  const { proc } = props;
  const status = proc.status ?? '';
  return <Tag {...(STATUS_LOOK[status] ?? UNKNOWN_STATUS_LOOK)} title={statusTip(proc)}>{status}</Tag>;
};

export { StatusCell };
