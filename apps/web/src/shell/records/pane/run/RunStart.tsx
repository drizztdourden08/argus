/* @layer renderer-app @kind component */
import { Button, Flex, Icon } from '@drizztdourden08/tessera';
import { useState } from 'react';
import type { RunStartProps } from './run.type';

const RunStart = (props: RunStartProps) => {
  const { ops, again } = props;
  const [busy, setBusy] = useState(false);
  const start = () => {
    setBusy(true);
    void ops.start().finally(() => setBusy(false));
  };
  return (
    <Flex gap="sm" align="center" wrap>
      <Button size="sm" icon={<Icon name="play" />} loading={busy} onClick={start}>{again ? 'Start a new run' : 'Start run'}</Button>
    </Flex>
  );
};

export { RunStart };
