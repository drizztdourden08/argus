/* @layer renderer-app @kind component */
import { Flex, Inline, Tag, Text } from '@drizztdourden08/tessera';
import { FAULT_LOOK, WAKING_LOOK } from './shell.constants';
import type { RegisteredFrameProps } from './shell.type';

const RegisteredFrame = (props: RegisteredFrameProps) => {
  const { label, rows, children } = props;
  if (rows.length === 0) return null;
  const waking = rows.filter((row) => row.state === 'waking').length;
  const faults = rows.filter((row) => row.state === 'fault').length;
  return (
    <Inline gap="md" align="center" wrap role="status" aria-label={label}>
      <Text variant="label">{`Registered ${rows.length}`}</Text>
      {waking > 0 && <Tag {...WAKING_LOOK}>{`${waking} waking`}</Tag>}
      {faults > 0 && <Tag {...FAULT_LOOK}>{`${faults} fault${faults > 1 ? 's' : ''}`}</Tag>}
      <Flex gap="md" wrap>{children}</Flex>
    </Inline>
  );
};

export { RegisteredFrame };
