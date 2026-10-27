/* @layer renderer-app @kind component */
import { Flex, Inline, Text } from '@drizztdourden08/tessera';
import type { RegisteredFrameProps } from './shell.type';
import './RegisteredFrame.css';

const RegisteredFrame = (props: RegisteredFrameProps) => {
  const { label, rows, children } = props;
  if (rows.length === 0) return null;
  const waking = rows.filter((row) => row.state === 'waking').length;
  const faults = rows.filter((row) => row.state === 'fault').length;
  return (
    <Inline className="registered-frame" gap="sm" align="center" wrap role="status" aria-label={label}>
      <Text variant="caption" tone="dim" weight={600} title="The threads hooked on this dashboard">
        {`Registered ${rows.length}`}
        {waking > 0 && <> · <Text variant="caption" tone="warning" weight={600}>{`${waking} waking`}</Text></>}
        {faults > 0 && <> · <Text variant="caption" tone="danger" weight={600}>{`${faults} fault${faults > 1 ? 's' : ''}`}</Text></>}
      </Text>
      <Flex gap="xs" wrap>{children}</Flex>
    </Inline>
  );
};

export { RegisteredFrame };
