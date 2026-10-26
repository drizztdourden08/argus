/* @layer renderer-app @kind component */
import { Button, Inline, Text } from '@drizztdourden08/tessera';
import type { TestPingProps } from '../notifications.type';

const TestPingBar = (props: TestPingProps) => {
  const { ping } = props;
  return (
    <Inline gap="md" align="center" wrap>
      <Button disabled={ping.busy} onClick={ping.send} title="Sends a ping to you, so you can see that the OS toast works">
        Send a test notification
      </Button>
      <Text role="status" aria-live="polite" tone="muted">{ping.message}</Text>
    </Inline>
  );
};

export { TestPingBar };
