/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { parseRecord } from '../behavior/parse-record';
import type { RecordLinkProps } from '../records.type';
import { RecordAnchor } from './RecordAnchor';

const RecordLink = (props: RecordLinkProps) => {
  const { id, board, children } = props;
  const parsed = parseRecord(id);
  const target = parsed && { ...parsed, board: parsed.board ?? board ?? null };
  const label = children ?? <Text as="span" variant="caption" leading="16" tone="muted" mono>{id}</Text>;
  return <RecordAnchor target={target} label={label} title={`Open ${id}`} />;
};

export { RecordLink };
