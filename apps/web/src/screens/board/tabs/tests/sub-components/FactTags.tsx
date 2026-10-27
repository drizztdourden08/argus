/* @layer renderer-app @kind component */
import { DateText, Status, Tag, Text } from '@drizztdourden08/tessera';
import { TEST_TAGS } from '../tests.constants';
import type { TestFactsProps } from '../tests-drawer-props.type';

const FactTags = (props: TestFactsProps) => {
  const { test } = props;
  const manual = test.mode === 'manual';
  return (
    <>
      <Tag {...(TEST_TAGS[test.status] ?? TEST_TAGS.pending)}>{test.status === 'pending' ? 'to run' : test.status}</Tag>
      {test.archivedAt && <Status tone="neutral" dot>archived</Status>}
      <Tag variant="urgency" color={manual ? 'warning' : 'info'}>{manual ? 'manual: needs you' : 'auto'}</Tag>
      {test.item && <Tag title={`Item ${test.item}`}>{test.item}</Tag>}
      <Text variant="caption" tone="muted">
        {test.thread ? `${test.thread} · ` : ''}updated <DateText value={test.updatedAt} format="elapsed" length="short" /> ago
      </Text>
    </>
  );
};

export { FactTags };
