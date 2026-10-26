/* @layer renderer-app @kind component */
import { Button, Flex, MarkdownEditor } from '@drizztdourden08/tessera';
import { useState } from 'react';
import type { DescriptionEditorProps } from '../kanban-props.type';
import { LIMITS, textProblem } from '../model';

const DescriptionEditor = (props: DescriptionEditorProps) => {
  const { value, onSave, onCancel } = props;
  const [draft, setDraft] = useState(value);
  const [problem, setProblem] = useState<string | null>(null);
  const save = () => {
    const found = textProblem('description', draft);
    setProblem(found);
    if (!found) onSave(draft.trim());
  };
  return (
    <Flex
      direction="column"
      gap="sm"
      onKeyDown={(event) => {
        if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
          event.preventDefault();
          save();
        } else if (event.key === 'Escape') {
          event.preventDefault();
          onCancel();
        }
      }}
    >
      <MarkdownEditor
        aria-label="Description"
        value={draft}
        onChange={(next) => {
          setProblem(null);
          setDraft(next);
        }}
        rows={6}
        hint={`Ctrl+Enter saves, Esc cancels, ${LIMITS.description} characters at most`}
        error={problem}
      />
      <Flex gap="xs">
        <Button size="sm" variant="primary" onClick={save}>Save description</Button>
        <Button size="sm" variant="ghost" onClick={onCancel}>Cancel</Button>
      </Flex>
    </Flex>
  );
};

export { DescriptionEditor };
