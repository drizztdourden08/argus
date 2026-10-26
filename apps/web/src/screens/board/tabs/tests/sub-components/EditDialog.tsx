/* @layer renderer-app @kind component */
import { Button, DialogShell, Flex, Select, Text, TextInput } from '@drizztdourden08/tessera';
import { useEditDraft } from '../behavior/useEditDraft';
import { useTestsShared } from '../behavior/useTestsShared';
import { EDIT_LIMITS } from '../model';
import { MODE_OPTIONS } from '../tests.constants';
import type { EditDialogProps } from '../tests-drawer-props.type';

const EditDialog = (props: EditDialogProps) => {
  const { test, onClose } = props;
  const { ops, items } = useTestsShared();
  const draft = useEditDraft(test, ops, onClose);
  const choices = items.filter((item) => !item.deletedAt).map((item) => ({ value: item.id, label: `${item.id} ${item.title.slice(0, 40)}` }));
  const actions = (
    <>
      <Button variant="ghost" onClick={onClose}>Cancel</Button>
      <Button variant="primary" onClick={draft.save}>Save</Button>
    </>
  );
  return (
    <DialogShell open onClose={onClose} title={`Edit ${test.id}`} size="md" actions={actions}>
      <Flex direction="column" gap="sm">
        <TextInput
          aria-label="Title"
          maxLength={EDIT_LIMITS.title}
          value={draft.value.title}
          invalid={draft.problem !== null}
          onChange={(event) => draft.set('title', event.target.value)}
          onEnter={draft.save}
          autoFocus
        />
        <Select aria-label="Mode" size="sm" options={MODE_OPTIONS} value={draft.value.mode} onChange={(value) => draft.set('mode', value)} />
        <Select aria-label="Item" size="sm" options={choices} value={draft.item} onChange={draft.setItem} />
        <TextInput aria-label="Covers" placeholder="Covers" maxLength={EDIT_LIMITS.covers} value={draft.value.covers} onChange={(event) => draft.set('covers', event.target.value)} />
        <TextInput aria-label="Build" placeholder="Build" maxLength={EDIT_LIMITS.build} value={draft.value.build} onChange={(event) => draft.set('build', event.target.value)} />
        {draft.problem && <Text variant="caption" tone="danger" role="alert">{draft.problem}</Text>}
      </Flex>
    </DialogShell>
  );
};

export { EditDialog };
