/* @layer renderer-app @kind component */
import { Pressable, Text } from '@drizztdourden08/tessera';
import { BoardMarkdown } from '../../../../shell';
import { useState } from 'react';
import type { MouseEvent } from 'react';
import type { EditableMarkdownProps } from '../shared.type';
import { MarkdownFieldEditor } from './MarkdownFieldEditor';

const onText = (event: MouseEvent<HTMLElement>): boolean => !(event.target as HTMLElement).closest('a, .markdown__image-button');

const EditableMarkdown = (props: EditableMarkdownProps) => {
  const { noun, value, empty, limit, problem, media, onSave } = props;
  const [editing, setEditing] = useState(false);
  if (editing) {
    return (
      <MarkdownFieldEditor
        noun={noun}
        value={value}
        limit={limit}
        problem={problem}
        media={media}
        onCancel={() => setEditing(false)}
        onSave={(next) => {
          setEditing(false);
          if (next !== value) onSave(next);
        }}
      />
    );
  }
  return (
    <Pressable aria-label={`Edit ${noun}`} title={`Click to edit the ${noun}`} onClick={(event) => onText(event) && setEditing(true)}>
      {value ? <BoardMarkdown pictures={media?.pictures}>{value}</BoardMarkdown> : <Text tone="muted" italic>{empty}</Text>}
    </Pressable>
  );
};

export { EditableMarkdown };
