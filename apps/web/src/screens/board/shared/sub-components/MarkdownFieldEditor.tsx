/* @layer renderer-app @kind component */
import { Button, Flex, MarkdownEditor } from '@drizztdourden08/tessera';
import { useState } from 'react';
import type { MarkdownFieldEditorProps } from '../shared.type';

const label = (noun: string): string => `${noun.charAt(0).toUpperCase()}${noun.slice(1)}`;

const MarkdownFieldEditor = (props: MarkdownFieldEditorProps) => {
  const { noun, value, limit, problem, onSave, onCancel } = props;
  const [draft, setDraft] = useState(value);
  const [found, setFound] = useState<string | null>(null);
  const save = (text: string) => {
    const next = problem(text);
    setFound(next);
    if (!next) onSave(text.trim());
  };
  return (
    <Flex direction="column" gap="sm">
      <MarkdownEditor
        aria-label={label(noun)}
        value={draft}
        onChange={(next) => {
          setFound(null);
          setDraft(next);
        }}
        rows={6}
        maxLength={limit}
        hint={`Ctrl+Enter saves, Esc cancels, ${limit} characters at most`}
        error={found}
        onSubmit={save}
        onEscape={onCancel}
      />
      <Flex gap="xs">
        <Button size="sm" variant="primary" onClick={() => save(draft)}>{`Save ${noun}`}</Button>
        <Button size="sm" variant="ghost" onClick={onCancel}>Cancel</Button>
      </Flex>
    </Flex>
  );
};

export { MarkdownFieldEditor };
