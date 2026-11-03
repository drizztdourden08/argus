/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { parseRecord } from '../behavior/parse-record';
import { useShownCode } from '../behavior/useShownCode';
import type { RecordLinkProps } from '../records.type';
import { RecordAnchor } from './RecordAnchor';

const RecordLink = (props: RecordLinkProps) => {
  const { id, board, tab, children } = props;
  const parsed = parseRecord(id);
  const shown = useShownCode(id, board);
  const target = parsed && { ...parsed, board: parsed.board ?? board ?? null, tab };
  const label = children ?? <Text as="span" variant="caption" leading="16" tone="muted" mono>{shown}</Text>;
  return <RecordAnchor target={target} label={label} title={`Open ${shown}`} />;
};

export { RecordLink };
