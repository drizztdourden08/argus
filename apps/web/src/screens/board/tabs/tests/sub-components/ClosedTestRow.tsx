/* @layer renderer-app @kind component */
import './TestRow.css';
import { Box, Text } from '@drizztdourden08/tessera';
import type { DoneSummaryProps } from '../tests-props.type';
import { DoneSummary } from './DoneSummary';

const ClosedTestRow = (props: DoneSummaryProps) => {
  const { test } = props;
  return (
    <Box className="test-row test-row--done" data-test={test.id}>
      <Text mono variant="caption" tone="muted" className="test-row__id">{test.id}</Text>
      <Box className="test-row__body">
        <Text weight={500} className="test-row__title" title={test.title} smartTimes>{test.title}</Text>
      </Box>
      <Box className="test-row__result"><DoneSummary test={test} /></Box>
    </Box>
  );
};

export { ClosedTestRow };
