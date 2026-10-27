/* @layer renderer-app @kind component */
import { InlineCreateForm, Select, toast } from '@drizztdourden08/tessera';
import { threadChoices } from '../../../shared';
import { useAddItemDraft } from '../behavior/useAddItemDraft';
import type { ColumnAddFormProps } from '../kanban-props.type';
import { textProblem } from '../model';

const ColumnAddForm = (props: ColumnAddFormProps) => {
  const { preset, group, threads, onCreate, onClose } = props;
  const draft = useAddItemDraft(threads, preset);
  const create = (title: string) => {
    const problem = textProblem('title', title);
    if (problem) {
      toast({ message: problem, variant: 'danger' });
      return;
    }
    onCreate(draft.itemOf(title));
    onClose();
  };
  return (
    <InlineCreateForm
      dashed
      label="Title"
      placeholder="New work item title"
      submitLabel="Add"
      canSubmit={draft.thread !== ''}
      onCreate={create}
      onCancel={onClose}
      extraFields={
        group === 'thread' ? undefined : <Select aria-label="Thread" size="sm" options={threadChoices(threads)} value={draft.thread} onChange={draft.setThread} />
      }
    />
  );
};

export { ColumnAddForm };
