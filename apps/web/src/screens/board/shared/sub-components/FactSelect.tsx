/* @layer renderer-app @kind component */
import { Flex, Select, Text } from '@drizztdourden08/tessera';
import { FIT_FIELD } from '../shared.constants';
import type { FactSelectProps } from '../shared.type';
import './FieldFit.css';

const FactSelect = (props: FactSelectProps) => {
  const { label, options, value, onPick } = props;
  return (
    <Flex gap="xs" align="center">
      <Text variant="caption" tone="muted">{label}</Text>
      <Select aria-label={label} size="sm" className={FIT_FIELD} options={options} value={value} onChange={(next) => next !== value && onPick(next)} />
    </Flex>
  );
};

export { FactSelect };
