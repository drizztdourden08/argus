/* @layer renderer-app @kind component */
import { Flex, Select, Text } from '@drizztdourden08/tessera';
import type { FactSelectProps } from '../drawer-props.type';

const FactSelect = (props: FactSelectProps) => {
  const { label, options, value, onPick } = props;
  return (
    <Flex gap="xs" align="center">
      <Text variant="caption" tone="muted">{label}</Text>
      <Select aria-label={label} size="sm" grow={false} options={options} value={value} onChange={(next) => next !== value && onPick(next)} />
    </Flex>
  );
};

export { FactSelect };
