/* @layer renderer-app @kind component */
import { Button, Flex, Text } from '@drizztdourden08/tessera';
import { useState } from 'react';
import type { EndRunProps } from './run.type';

const EndRun = (props: EndRunProps) => {
  const { left, ops } = props;
  const [busy, setBusy] = useState(false);
  const end = () => {
    setBusy(true);
    void ops.end().finally(() => setBusy(false));
  };
  return (
    <Flex gap="sm" align="center" justify="end" wrap>
      {left > 0 && <Text variant="caption" tone="muted">{`${left} not run yet; they count as not run.`}</Text>}
      <Button size="sm" variant={left === 0 ? 'primary' : 'secondary'} loading={busy} onClick={end}>End run</Button>
    </Flex>
  );
};

export { EndRun };
