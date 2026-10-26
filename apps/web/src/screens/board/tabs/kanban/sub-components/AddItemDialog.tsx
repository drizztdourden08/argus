/* @layer renderer-app @kind component */
import { Dialog, Flex, Select, TextInput } from '@drizztdourden08/tessera';
import { useAddItemDraft } from '../behavior/useAddItemDraft';
import type { AddItemDialogProps } from '../kanban-props.type';
import { LIMITS, STATUS_OPTIONS, threadChoices } from '../model';

const AddItemDialog = (props: AddItemDialogProps) => {
  const { open, threads, onCreate, onClose } = props;
  const draft = useAddItemDraft(threads, (item) => {
    onCreate(item);
    onClose();
  });
  const cancel = () => {
    draft.reset();
    onClose();
  };
  return (
    <Dialog
      open={open}
      title="New work item"
      message="Add an item to the board. The thread it names takes it from there."
      confirmLabel="Add"
      confirmDisabled={!draft.ready}
      onConfirm={draft.submit}
      onCancel={cancel}
    >
      <Flex direction="column" gap="sm">
        <TextInput
          aria-label="Title"
          placeholder="New work item title"
          maxLength={LIMITS.title}
          value={draft.title}
          onChange={(event) => draft.setTitle(event.target.value)}
          onEnter={draft.submit}
          autoFocus
        />
        <Select aria-label="Thread" options={threadChoices(threads)} value={draft.thread} onChange={draft.setThread} />
        <Select aria-label="Status" options={STATUS_OPTIONS} value={draft.status} onChange={draft.setStatus} />
      </Flex>
    </Dialog>
  );
};

export { AddItemDialog };
