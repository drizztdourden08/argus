/* @layer renderer-app @kind component */
import { DateText, Flex, Tag, Text } from '@drizztdourden08/tessera';
import { byLabel } from '../model';
import { TEST_TAGS } from '../tests.constants';
import type { ResultBlockProps } from '../tests-drawer-props.type';

const ResultMeta = (props: ResultBlockProps) => {
  const { test } = props;
  const { result } = test;
  const pending = test.status === 'pending';
  const status = pending ? 'pending' : result?.status ?? test.status;
  return (
    <Flex gap="sm" align="center" wrap>
      <Tag {...(TEST_TAGS[status] ?? TEST_TAGS.pending)}>{pending ? 'to run' : status}</Tag>
      {result && (
        <Text variant="caption" tone="muted">
          {pending ? 'last result ' : ''}<DateText value={result.at} format="elapsed" length="short" /> ago by {byLabel(result.by)}
        </Text>
      )}
    </Flex>
  );
};

export { ResultMeta };
