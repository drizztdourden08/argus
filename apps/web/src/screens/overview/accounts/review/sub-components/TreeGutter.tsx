/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import type { TreeGutterProps } from '../review-tree-props.type';
import { GutterCell } from './GutterCell';

const TreeGutter = (props: TreeGutterProps) => {
  const { pair, rows, hovered, env } = props;
  return (
    <Box className="review-tree-gutter" role="group" aria-label="Choices">
      <Box className="review-tree-head">Choice</Box>
      {rows.map((row) => <GutterCell key={row.id} row={row} pair={pair} hot={hovered === row.id} env={env} />)}
    </Box>
  );
};

export { TreeGutter };
