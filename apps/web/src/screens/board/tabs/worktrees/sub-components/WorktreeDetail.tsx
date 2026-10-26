/* @layer renderer-app @kind component */
import { isLeftover, leftoverLimit } from '../model';
import type { DetailProps } from '../worktrees-props.type';
import { LeftoverConfirm } from './LeftoverConfirm';
import { RemoveConfirm } from './RemoveConfirm';
import { RequestForm } from './RequestForm';

const WorktreeDetail = (props: DetailProps) => {
  const { row, meta, controls, ask, removal } = props;
  const { panel } = controls;
  if (panel?.id !== row.id) return null;
  if (panel.kind !== 'remove') return <RequestForm row={row} panel={panel} meta={meta} controls={controls} ask={ask} />;
  if (isLeftover(row)) return <LeftoverConfirm row={row} panel={panel} limit={leftoverLimit(meta)} controls={controls} removal={removal} />;
  return <RemoveConfirm row={row} panel={panel} controls={controls} removal={removal} />;
};

export { WorktreeDetail };
