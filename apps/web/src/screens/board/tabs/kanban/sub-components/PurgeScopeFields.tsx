/* @layer renderer-app @kind component */
import { Checkbox, Flex, NumberInput, Select } from '@drizztdourden08/tessera';
import type { PurgeScopeFieldsProps } from '../trash-props.type';
import { threadChoices } from '../../../shared';
import { PURGE_STATUSES } from '../trash';

const toggled = (list: readonly string[], value: string, on: boolean): string[] => (on ? [...list, value] : list.filter((entry) => entry !== value));

const PurgeScopeFields = (props: PurgeScopeFieldsProps) => {
  const { draft, threads, patch } = props;
  const owners = threadChoices(threads.filter((thread) => thread.role !== 'coordinator'));
  if (draft.scope === 'thread') {
    return (
      <Select
        aria-label="Thread to purge"
        placeholder="Choose a thread"
        options={owners}
        value={draft.thread || undefined}
        onChange={(thread) => patch({ thread })}
      />
    );
  }
  if (draft.scope === 'keep') {
    return (
      <Flex direction="column" gap="xs" role="group" aria-label="Threads to keep">
        {owners.map((owner) => (
          <Checkbox
            key={owner.value}
            label={owner.label}
            checked={draft.keep.includes(owner.value)}
            onChange={(on) => patch({ keep: toggled(draft.keep, owner.value, on) })}
          />
        ))}
      </Flex>
    );
  }
  if (draft.scope === 'status') {
    return (
      <Flex gap="md" wrap role="group" aria-label="Thread statuses">
        {PURGE_STATUSES.map((status) => (
          <Checkbox key={status} label={status} checked={draft.status.includes(status)} onChange={(on) => patch({ status: toggled(draft.status, status, on) })} />
        ))}
      </Flex>
    );
  }
  if (draft.scope === 'idle') {
    return <NumberInput aria-label="Days idle" min={1} max={3650} value={draft.idle} onChange={(idle) => patch({ idle })} />;
  }
  return null;
};

export { PurgeScopeFields };
