/* @layer renderer-app @kind component */
import { Flex, ListItem, Text } from '@drizztdourden08/tessera';
import type { DoneSummaryProps } from '../tests-props.type';
import { DoneSummary } from './DoneSummary';

const ClosedTestRow = (props: DoneSummaryProps) => {
  const { test } = props;
  return (
    <ListItem bare data-test={test.id}>
      <Flex gap="sm" align="baseline" wrap>
        <Text mono tone="muted">{test.id}</Text>
        <Text>{test.title}</Text>
        <DoneSummary test={test} />
      </Flex>
    </ListItem>
  );
};

export { ClosedTestRow };
