/* @layer renderer-app @kind component */
import { DateText, Flex, Markdown, Status, Text } from '@drizztdourden08/tessera';
import { byLabel } from '../model';
import { TEST_STATUSES } from '../tests.constants';
import type { ResultBlockProps } from '../tests-props.type';
import { DrawerSection } from './DrawerSection';

const ResultBlock = (props: ResultBlockProps) => {
  const { test } = props;
  const { result } = test;
  return (
    <DrawerSection title="Result">
      <Flex gap="sm" align="center" wrap>
        <Status map={TEST_STATUSES} value={test.status} dot />
        {result && (
          <Text variant="caption" tone="muted">
            {test.status === 'pending' ? 'last result ' : ''}<DateText value={result.at} format="relative" /> by {byLabel(result.by)}
          </Text>
        )}
      </Flex>
      {result?.text ? <Markdown size="sm">{result.text}</Markdown> : <Text tone="muted">Not run yet.</Text>}
    </DrawerSection>
  );
};

export { ResultBlock };
