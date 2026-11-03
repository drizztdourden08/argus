/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { useDep } from '../../../data/deps';
import { parseRecord } from '../behavior/parse-record';
import { DEP_TAB } from '../records.constants';
import type { DepRecordLinkProps, RecordRef } from '../records.type';
import { RecordAnchor } from './RecordAnchor';

const DepRecordLink = (props: DepRecordLinkProps) => {
  const { id } = props;
  const { doc } = useDep(id);
  const end = doc?.from;
  const parsed = end?.item ? parseRecord(end.item) : null;
  const target: RecordRef | null = parsed && end ? { ...parsed, board: end.project, tab: DEP_TAB } : null;
  return <RecordAnchor target={target} label={<Text as="span" variant="caption" tone="muted" mono>{id}</Text>} title={`Open dependency ${id}`} />;
};

export { DepRecordLink };
