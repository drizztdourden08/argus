/* @layer renderer-app @kind component */
import { Dialog, Flex, RadioGroup, Text, TextInput } from '@drizztdourden08/tessera';
import { useMemo } from 'react';
import { useCollection } from '../../../../../data/react';
import type { LogDoc } from '../../../../../data/types';
import { usePurge } from '../behavior/usePurge';
import { usePurgeDraft } from '../behavior/usePurgeDraft';
import { PURGE_SCOPES } from '../kanban.constants';
import type { PurgeDialogProps } from '../trash-props.type';
import { purgeCounts, purgeCriteria, purgeTotal } from '../trash';
import type { PurgeScope } from '../trash';
import { PurgeCountsView } from './PurgeCountsView';
import { PurgeScopeFields } from './PurgeScopeFields';

const PurgeDialog = (props: PurgeDialogProps) => {
  const { boardKey, data, focus, scope, onClose } = props;
  const log = useCollection<LogDoc>(boardKey, 'log');
  const send = usePurge(boardKey);
  const [draft, patch] = usePurgeDraft(data.threads.some((thread) => thread.id === focus && thread.role !== 'coordinator') ? focus : '', scope);
  const counts = useMemo(
    () => purgeCounts(
      purgeCriteria(draft),
      { threads: data.threads, items: data.items, tests: data.tests, actions: data.actions, entries: data.entries, log: log.docs },
      Date.now(),
    ),
    [draft, data, log.docs],
  );
  const total = purgeTotal(counts);
  const ready = total > 0 && draft.confirm === boardKey;
  return (
    <Dialog
      open
      size="lg"
      variant="danger"
      title="Purge"
      message="A purge removes records from the board and from this page for good. The dashboard first writes a restorable backup on this machine (the coordinator restores it with coord restore). The coordinator thread is never purged."
      confirmLabel="Purge"
      confirmDisabled={!ready}
      onConfirm={() => void send(draft, counts).then((ok) => ok && onClose())}
      onCancel={onClose}
    >
      <Flex direction="column" gap="md">
        <RadioGroup<PurgeScope> label="Scope" value={draft.scope} options={PURGE_SCOPES} onChange={(scope) => patch({ scope })} />
        <PurgeScopeFields draft={draft} threads={data.threads} patch={patch} />
        <PurgeCountsView counts={counts} />
        <Text variant="caption" tone="muted">{total === 0 ? 'Nothing matches. Nothing to purge.' : ''}</Text>
        <TextInput
          aria-label={`Type the project key ${boardKey} to confirm`}
          placeholder={`Type ${boardKey} to confirm`}
          autoComplete="off"
          value={draft.confirm}
          onChange={(event) => patch({ confirm: event.target.value.trim() })}
        />
      </Flex>
    </Dialog>
  );
};

export { PurgeDialog };
