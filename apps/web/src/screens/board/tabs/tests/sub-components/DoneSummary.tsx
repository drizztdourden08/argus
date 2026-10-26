/* @layer renderer-app @kind component */
import { DateText, Flex, Status, Text } from '@drizztdourden08/tessera';
import { byLabel } from '../model';
import { RESULT_PLACEHOLDER, TEST_STATUSES } from '../tests.constants';
import type { DoneSummaryProps } from '../tests-props.type';

const DoneSummary = (props: DoneSummaryProps) => {
  const { test } = props;
  const { result } = test;
  const note = result?.text && result.text !== RESULT_PLACEHOLDER ? result.text : '';
  return (
    <Flex direction="column" gap="2xs">
      <Flex gap="sm" align="center" wrap>
        <Status map={TEST_STATUSES} value={test.status} dot />
        <Text variant="caption" tone="muted">{byLabel(result?.by)} / <DateText value={result?.at ?? test.updatedAt} format="relative" /></Text>
      </Flex>
      {note && <Text variant="caption" tone="dim">{note}</Text>}
    </Flex>
  );
};

export { DoneSummary };
