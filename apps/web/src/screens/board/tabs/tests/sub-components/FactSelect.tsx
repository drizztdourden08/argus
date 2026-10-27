/* @layer renderer-app @kind component */
import './TestsFit.css';
import { Flex, Select, Text } from '@drizztdourden08/tessera';
import type { FactSelectProps } from '../tests-drawer-props.type';

const FactSelect = (props: FactSelectProps) => {
  const { label, options, value, onPick } = props;
  return (
    <Flex gap="xs" align="center">
      <Text variant="caption" tone="muted">{label}</Text>
      <Select aria-label={label} size="sm" className="tests-fit" options={options} value={value} onChange={onPick} />
    </Flex>
  );
};

export { FactSelect };
