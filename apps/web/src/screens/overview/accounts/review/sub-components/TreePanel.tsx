/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import type { TreePanelProps } from '../review-tree-props.type';
import { TreeRow } from './TreeRow';

const TreePanel = (props: TreePanelProps) => {
  const { account, rows, hovered, env } = props;
  const name = account.label ?? account.key;
  return (
    <Box className="review-tree-panel" role="group" aria-label={name}>
      <Box className="review-tree-head">{name}</Box>
      {rows.map((row) => <TreeRow key={row.id} row={row} account={account.key} hot={hovered === row.id} env={env} />)}
    </Box>
  );
};

export { TreePanel };
