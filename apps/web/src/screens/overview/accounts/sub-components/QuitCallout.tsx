/* @layer renderer-app @kind component */
import { Button, Callout, Stack, Text } from '@drizztdourden08/tessera';
import type { AccountsRun } from '../../../../data/types';
import { useQuitCancel } from '../behavior/useQuitCancel';
import { QUIT_CANCEL_LABEL, QUIT_FALLBACK } from '../compare.constants';

const QuitCallout = (props: { run: AccountsRun | null }) => {
  const cancel = useQuitCancel();
  if (props.run?.phase !== 'waiting_for_quit') return null;
  return (
    <Callout tone="warning">
      <Stack gap="sm">
        <Text>{props.run.detail ?? QUIT_FALLBACK}</Text>
        <Button size="sm" variant="secondary" disabled={cancel.busy} onClick={cancel.run}>{QUIT_CANCEL_LABEL}</Button>
      </Stack>
    </Callout>
  );
};

export { QuitCallout };
