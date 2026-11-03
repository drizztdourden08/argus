/* @layer renderer-app @kind component */
import { Flex, OutcomeStrip, Tag, Text } from '@drizztdourden08/tessera';
import type { RunHeadProps } from './run.type';

const RunHead = (props: RunHeadProps) => {
  const { run, position, total, steps, onSelect } = props;
  return (
    <Flex direction="column" gap="sm">
      <Flex gap="sm" align="center" wrap>
        <Text as="span" weight={600}>{`Run ${run.n}`}</Text>
        {run.bundle && <Tag color="neutral" title={`Bundle ${run.bundle}`}>{run.bundle}</Tag>}
        <Text as="span" variant="caption" tone="muted">{`case ${position} of ${total}`}</Text>
      </Flex>
      <OutcomeStrip aria-label="Cases of this run" steps={steps} size="sm" onSelect={onSelect} />
    </Flex>
  );
};

export { RunHead };
