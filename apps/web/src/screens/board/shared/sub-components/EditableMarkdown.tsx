/* @layer renderer-app @kind component */
import { Button, Flex, Text } from '@drizztdourden08/tessera';
import { BoardMarkdown } from '../../../../shell';
import { useState } from 'react';
import type { EditableMarkdownProps } from '../shared.type';
import { MarkdownFieldEditor } from './MarkdownFieldEditor';

const EditableMarkdown = (props: EditableMarkdownProps) => {
  const { noun, value, empty, limit, problem, onSave } = props;
  const [editing, setEditing] = useState(false);
  if (editing) {
    return (
      <MarkdownFieldEditor
        noun={noun}
        value={value}
        limit={limit}
        problem={problem}
        onCancel={() => setEditing(false)}
        onSave={(next) => {
          setEditing(false);
          if (next !== value) onSave(next);
        }}
      />
    );
  }
  return (
    <Flex direction="column" gap="xs" align="start">
      {value ? <BoardMarkdown>{value}</BoardMarkdown> : <Text tone="muted" italic>{empty}</Text>}
      <Button size="xs" variant="ghost" onClick={() => setEditing(true)}>{`Edit ${noun}`}</Button>
    </Flex>
  );
};

export { EditableMarkdown };
