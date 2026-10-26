/* @layer renderer-app @kind component */
import { Status, Tag } from '@drizztdourden08/tessera';
import { useTestsShared } from '../behavior/useTestsShared';
import { threadName } from '../model';
import { TEST_STATUSES } from '../tests.constants';
import type { TestFactsProps } from '../tests-drawer-props.type';

const FactTags = (props: TestFactsProps) => {
  const { test, handed } = props;
  const { threads } = useTestsShared();
  const manual = test.mode === 'manual';
  return (
    <>
      <Status map={TEST_STATUSES} value={test.status} variant="pill" dot />
      <Tag color={manual ? 'primary' : 'neutral'}>{manual ? 'manual: needs you' : 'auto'}</Tag>
      {test.item && <Tag title={`Item ${test.item}`}>{test.item}</Tag>}
      {test.thread && <Tag title={`Thread ${test.thread}`}>{threadName(threads, test.thread)}</Tag>}
      {test.build && <Tag title="The build this test runs on">{test.build}</Tag>}
      {handed && <Tag color="primary">handed to you</Tag>}
      {test.archivedAt && <Status tone="neutral" dot>archived</Status>}
    </>
  );
};

export { FactTags };
