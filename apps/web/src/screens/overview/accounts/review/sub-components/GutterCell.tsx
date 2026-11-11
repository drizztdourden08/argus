/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import { scopeOf } from '../model';
import type { GutterCellProps } from '../review-tree-props.type';
import { ChoiceMenu } from './ChoiceMenu';
import { SideButton } from './SideButton';
import { SlotGap } from './SlotGap';

const GutterCell = (props: GutterCellProps) => {
  const { row, pair, hot, env } = props;
  const rows = row.kind === 'folder' ? scopeOf(row, env.scopes) : [row];
  const className = hot ? 'review-tree-gutter-cell review-tree-gutter-cell--hot' : 'review-tree-gutter-cell';
  const hover = { onMouseEnter: () => env.onHover(row.id), onMouseLeave: () => env.onHover(null) };
  if (!rows.some((item) => item.diff)) return <Box className={className} {...hover} />;
  return (
    <Box className={className} {...hover}>
      <SideButton side="left" rows={rows} pair={pair} env={env} />
      <SideButton side="skip" rows={rows} pair={pair} env={env} />
      <SideButton side="right" rows={rows} pair={pair} env={env} />
      {rows.length === 1 ? <ChoiceMenu row={row} pair={pair} env={env} /> : <SlotGap />}
    </Box>
  );
};

export { GutterCell };
