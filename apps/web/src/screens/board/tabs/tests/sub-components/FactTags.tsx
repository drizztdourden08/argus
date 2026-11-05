/* @layer renderer-app @kind component */
import { Status, Tag, Text } from '@drizztdourden08/tessera';
import { labelOf } from '../../../../../data/registry';
import { Age, RecordLink, ShownCode } from '../../../../../shell';
import { TEST_TAGS } from '../tests.constants';
import type { TestFactsProps } from '../tests-drawer-props.type';

const FactTags = (props: TestFactsProps) => {
  const { test } = props;
  const manual = test.mode === 'manual';
  return (
    <>
      <Tag {...(TEST_TAGS[test.status] ?? TEST_TAGS.pending)}>{test.status === 'pending' ? 'to run' : test.status}</Tag>
      {test.archivedAt && <Status tone="neutral" dot>archived</Status>}
      <Tag variant="urgency" color={manual ? 'warning' : 'info'}>{labelOf('mode', manual ? 'manual' : 'auto')}</Tag>
      {test.item && <RecordLink id={test.item}><Tag title={`Item ${test.item}`}><ShownCode id={test.item} /></Tag></RecordLink>}
      <Text variant="caption" tone="muted">
        {test.thread ? `${test.thread} · ` : ''}updated <Age value={test.updatedAt} /> ago
      </Text>
    </>
  );
};

export { FactTags };
