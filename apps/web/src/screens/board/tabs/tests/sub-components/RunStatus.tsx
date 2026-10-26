/* @layer renderer-app @kind component */
import { DateText, Flex, Status, Text } from '@drizztdourden08/tessera';
import { runLine } from '../model';
import type { RunStatusProps } from '../tests-props.type';

const RunStatus = (props: RunStatusProps) => {
  const { run } = props;
  const line = runLine(run);
  return (
    <Flex gap="xs" align="center" wrap role="status" aria-label={`Run ${run.id}`}>
      <Status tone={line.tone} dot pulse={run.status === 'running'}>{line.label}</Status>
      {line.since && (
        <Text variant="caption" tone="muted">
          {run.status === 'running' ? 'since ' : ''}
          <DateText value={line.since} format="relative" />
        </Text>
      )}
      {line.last && <Text variant="caption" tone="dim" mono>{line.last}</Text>}
    </Flex>
  );
};

export { RunStatus };
