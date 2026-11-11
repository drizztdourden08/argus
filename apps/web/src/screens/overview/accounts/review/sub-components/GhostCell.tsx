/* @layer renderer-app @kind component */
import { Inline, Text } from '@drizztdourden08/tessera';
import { INDENT } from '../../compare.constants';
import { cellNote, choiceOf } from '../model';
import { NO_FOLDER } from '../review.constants';
import type { TreeCellProps } from '../review-props.type';

const GhostCell = (props: TreeCellProps) => {
  const { row, account, env } = props;
  const note = cellNote(choiceOf(row, env.decisions), account.key, { present: false }) ?? 'not created';
  return (
    <Inline gap="xs" align="center">
      <Text as="span">{INDENT.repeat(row.depth + 2)}</Text>
      <Text italic tone="muted">{`+ ${row.title || NO_FOLDER}`}</Text>
      <Text variant="caption" tone="muted">{note}</Text>
    </Inline>
  );
};

export { GhostCell };
