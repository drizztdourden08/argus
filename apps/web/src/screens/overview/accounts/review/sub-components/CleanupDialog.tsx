/* @layer renderer-app @kind component */
import { Button, Callout, Dialog, Inline, NumberInput, Spinner, Stack, Text } from '@drizztdourden08/tessera';
import { useCleanup } from '../behavior/useCleanup';
import { CLEANUP_NOTE } from '../review.constants';
import type { CleanupDialogProps } from '../review-props.type';
import { CleanupList } from './CleanupList';

const CleanupDialog = (props: CleanupDialogProps) => {
  const { accounts, onStage, onCancel } = props;
  const cleanup = useCleanup();
  const chosen = cleanup.found?.candidates.filter((candidate) => cleanup.picked.has(`${candidate.account}|${candidate.sessionId}`)) ?? [];
  const stage = () => onStage(chosen.map((candidate) => ({ op: 'delete_chat', account: candidate.account, sessionId: candidate.sessionId })));
  return (
    <Dialog
      open
      size="md"
      title="Clean up archived chats"
      message={CLEANUP_NOTE}
      confirmLabel={`Stage ${chosen.length} deletes`}
      confirmDisabled={chosen.length === 0}
      onConfirm={stage}
      onCancel={onCancel}
    >
      <Stack gap="sm">
        <Inline gap="sm" align="center">
          <NumberInput size="sm" aria-label="Days untouched" min={1} value={cleanup.days} onChange={cleanup.setDays} />
          <Text>days</Text>
          <Button size="sm" variant="ghost" onClick={cleanup.loading ? undefined : cleanup.find}>Find</Button>
          {cleanup.loading && <Spinner size="sm" />}
        </Inline>
        {cleanup.error && <Callout tone="danger">{cleanup.error}</Callout>}
        {cleanup.found && <CleanupList accounts={accounts} found={cleanup.found} picked={cleanup.picked} now={Date.now()} onToggle={cleanup.toggle} />}
      </Stack>
    </Dialog>
  );
};

export { CleanupDialog };
