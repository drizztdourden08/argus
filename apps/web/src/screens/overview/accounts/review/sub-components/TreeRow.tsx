/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import { memo } from 'react';
import { rowView } from '../model';
import { NO_FOLDER } from '../review.constants';
import type { TreeRowProps } from '../review-tree-props.type';
import { CellNotes } from './CellNotes';
import { RowActions } from './RowActions';
import { RowTitle } from './RowTitle';
import { TreeLead } from './TreeLead';
import { TreeSteps } from './TreeSteps';

const rowClass = (hot: boolean, same: boolean): string => ['review-tree-row', hot && 'review-tree-row--hot', same && 'review-tree-row--same'].filter(Boolean).join(' ');

const TreeRowBase = (props: TreeRowProps) => {
  const { row, account, hot, env } = props;
  const view = rowView(row, account, env);
  const toggle = () => env.onToggle(row.id);
  return (
    <Box className={rowClass(hot, row.diff === null)} onMouseEnter={() => env.onHover(row.id)} onMouseLeave={() => env.onHover(null)}>
      <TreeSteps depth={row.depth} />
      <TreeLead row={row} ghost={!view.present} kids={view.kids} collapsed={view.collapsed} onToggle={toggle} />
      <RowTitle row={row} ghost={!view.present} muted={view.archived !== null} count={view.count} title={row.title || NO_FOLDER} onToggle={view.kids ? toggle : undefined} />
      <CellNotes archived={view.archived} note={view.note} pending={env.pending.has(`${account}|${row.id}`)} />
      {view.present && <RowActions row={row} account={account} env={env} />}
    </Box>
  );
};

const TreeRow = memo(TreeRowBase);

export { TreeRow };
