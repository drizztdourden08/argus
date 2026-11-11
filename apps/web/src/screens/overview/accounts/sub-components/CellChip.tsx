/* @layer renderer-app @kind component */
import { QUIET_STATUSES, STATUS_LOOK } from '../compare.constants';
import type { CellChipProps } from '../compare.type';
import { StatusChip } from './StatusChip';

const CellChip = (props: CellChipProps) => {
  const { status } = props;
  if (!status || QUIET_STATUSES.has(status)) return null;
  return <StatusChip look={STATUS_LOOK[status]} />;
};

export { CellChip };
