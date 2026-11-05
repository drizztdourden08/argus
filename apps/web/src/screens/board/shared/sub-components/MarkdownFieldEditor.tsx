/* @layer renderer-app @kind component */
import { Button, Flex } from '@drizztdourden08/tessera';
import { BoardMarkdownEditor } from '../../../../shell';
import { useState } from 'react';
import type { MarkdownFieldEditorProps } from '../shared.type';

const label = (noun: string): string => `${noun.charAt(0).toUpperCase()}${noun.slice(1)}`;

const MarkdownFieldEditor = (props: MarkdownFieldEditorProps) => {
  const { noun, value, limit, problem, media, onSave, onCancel } = props;
  const [draft, setDraft] = useState(value);
  const [found, setFound] = useState<string | null>(null);
  const save = (text: string) => {
    const next = problem(text);
    setFound(next);
    if (!next) onSave(text.trim());
  };
  return (
    <Flex direction="column" gap="xs">
      <BoardMarkdownEditor
        aria-label={label(noun)}
        value={draft}
        onChange={(next) => {
          setFound(null);
          setDraft(next);
        }}
        layout="tabs"
        rows={4}
        autoFocus
        maxLength={limit}
        count={media?.count}
        onImageFiles={media?.onImageFiles}
        pictures={media?.pictures}
        error={found}
        onSubmit={save}
        onEscape={onCancel}
        submitLabel="Save"
        actions={<Button size="sm" variant="ghost" onClick={onCancel}>Cancel</Button>}
      />
    </Flex>
  );
};

export { MarkdownFieldEditor };
