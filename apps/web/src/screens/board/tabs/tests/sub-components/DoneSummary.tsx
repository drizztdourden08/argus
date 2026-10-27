/* @layer renderer-app @kind component */
import { Flex, Tag, Text } from '@drizztdourden08/tessera';
import { Age } from '../../../../../shell';
import { byLabel } from '../model';
import { TEST_TAGS } from '../tests.constants';
import type { DoneSummaryProps } from '../tests-props.type';
import { WakeStatus } from './WakeStatus';

const DoneSummary = (props: DoneSummaryProps) => {
  const { test } = props;
  const { result } = test;
  return (
    <Flex gap="sm" align="center" wrap>
      <Tag {...(TEST_TAGS[test.status] ?? TEST_TAGS.pending)}>{test.status === 'pending' ? 'to run' : test.status}</Tag>
      <Text variant="caption" tone="muted">
        {byLabel(result?.by)} · <Age value={result?.at ?? test.updatedAt} /> ago
      </Text>
      <WakeStatus test={test} />
    </Flex>
  );
};

export { DoneSummary };
