/* @layer renderer-app @kind component */
import { Button, Flex, Text, TextInput } from '@drizztdourden08/tessera';
import { useInlineEdit } from '../behavior/useInlineEdit';
import type { InlineEditProps } from '../kanban-props.type';

const InlineEdit = (props: InlineEditProps) => {
  const { label, value, field, display, onSave } = props;
  const edit = useInlineEdit(value, field, onSave);
  if (edit.draft === null) {
    return (
      <Flex gap="xs" align="center" wrap>
        {display}
        <Button size="xs" variant="ghost" aria-label={`Edit ${label}`} onClick={edit.start}>Edit</Button>
      </Flex>
    );
  }
  return (
    <Flex direction="column" gap="xs">
      <Flex gap="xs" align="center">
        <TextInput
          aria-label={label}
          value={edit.draft}
          invalid={edit.problem !== null}
          onChange={(event) => edit.change(event.target.value)}
          onEnter={edit.commit}
          onKeyDown={(event) => {
            if (event.key !== 'Escape') return;
            event.preventDefault();
            edit.cancel();
          }}
          autoFocus
        />
        <Button size="xs" variant="primary" aria-label={`Save ${label}`} onClick={edit.commit}>Save</Button>
        <Button size="xs" variant="ghost" aria-label={`Cancel ${label}`} onClick={edit.cancel}>Cancel</Button>
      </Flex>
      {edit.problem && <Text variant="caption" tone="danger" role="alert">{edit.problem}</Text>}
    </Flex>
  );
};

export { InlineEdit };
