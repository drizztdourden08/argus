/* @layer renderer-app @kind component */
import { Flex, Select, Text } from '@drizztdourden08/tessera';
import type { FactSelectProps } from '../shared.type';
import { FieldFit } from './FieldFit';

const FactSelect = (props: FactSelectProps) => {
  const { label, options, value, onPick } = props;
  return (
    <Flex gap="xs" align="center">
      <Text variant="caption" tone="muted">{label}</Text>
      <FieldFit>
        <Select aria-label={label} size="sm" options={options} value={value} onChange={(next) => next !== value && onPick(next)} />
      </FieldFit>
    </Flex>
  );
};

export { FactSelect };
