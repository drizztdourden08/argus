/* @layer renderer-app @kind component */
import { Button, Flex, InlineEdit, Tag, Text } from '@drizztdourden08/tessera';
import type { TodoRowProps } from '../drawer-props.type';
import { validateField } from '../model';

const validateTodo = validateField('todo');

const TodoRow = (props: TodoRowProps) => {
  const { todo, ops, acts } = props;
  const { action, done } = todo;
  const text = action.text ?? '';
  const reopen = () => void ops.editRecord({ c: 'actions', id: action.id, rev: action.rev }, { field: 'status', prev: 'done', value: 'pending' }, 'To-do reopened.');
  return (
    <Flex gap="sm" align="center" wrap>
      <Text variant="caption" tone="muted" mono>{action.id}</Text>
      <InlineEdit
        label={`to-do ${action.id}`}
        placeholder="to-do text"
        value={text}
        validate={validateTodo}
        onCommit={(value) => void ops.editRecord({ c: 'actions', id: action.id, rev: action.rev }, { field: 'text', prev: text, value: value.trim() }, `Saved ${action.id}.`)}
      />
      {action.archivedAt && <Tag>archived</Tag>}
      {done ? (
        <Button size="xs" variant="ghost" aria-label={`Reopen ${action.id}`} onClick={reopen}>Reopen</Button>
      ) : (
        <Button size="xs" variant="primary" aria-label={`Mark ${action.id} done`} disabled={acts.isBusy(`post:done:${action.id}`)} onClick={() => void acts.done(action)}>
          Done
        </Button>
      )}
    </Flex>
  );
};

export { TodoRow };
