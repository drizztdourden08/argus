/* @layer renderer-app @kind component */
import { Button, Flex, MarkdownEditor } from '@drizztdourden08/tessera';
import { useState } from 'react';
import type { DescriptionEditorProps } from '../drawer-props.type';
import { LIMITS, textProblem } from '../model';

const DescriptionEditor = (props: DescriptionEditorProps) => {
  const { value, onSave, onCancel } = props;
  const [draft, setDraft] = useState(value);
  const [problem, setProblem] = useState<string | null>(null);
  const save = (text: string) => {
    const found = textProblem('description', text);
    setProblem(found);
    if (!found) onSave(text.trim());
  };
  return (
    <Flex direction="column" gap="sm">
      <MarkdownEditor
        aria-label="Description"
        value={draft}
        onChange={(next) => {
          setProblem(null);
          setDraft(next);
        }}
        rows={6}
        maxLength={LIMITS.description}
        hint={`Ctrl+Enter saves, Esc cancels, ${LIMITS.description} characters at most`}
        error={problem}
        onSubmit={save}
        onEscape={onCancel}
      />
      <Flex gap="xs">
        <Button size="sm" variant="primary" onClick={() => save(draft)}>Save description</Button>
        <Button size="sm" variant="ghost" onClick={onCancel}>Cancel</Button>
      </Flex>
    </Flex>
  );
};

export { DescriptionEditor };
