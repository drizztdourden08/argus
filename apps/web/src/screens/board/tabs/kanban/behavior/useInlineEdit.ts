/* @layer renderer-app @kind logic */
import { useCallback, useState } from 'react';
import { textProblem } from '../model';
import type { EditableField } from '../model';
import type { InlineEditState } from '../kanban.type';

const useInlineEdit = (value: string, field: EditableField, onSave: (value: string) => void): InlineEditState => {
  const [draft, setDraft] = useState<string | null>(null);
  const [problem, setProblem] = useState<string | null>(null);

  const start = useCallback(() => {
    setProblem(null);
    setDraft(value);
  }, [value]);

  const cancel = useCallback(() => {
    setProblem(null);
    setDraft(null);
  }, []);

  const change = useCallback((next: string) => {
    setProblem(null);
    setDraft(next);
  }, []);

  const commit = useCallback(() => {
    if (draft === null) return;
    const found = textProblem(field, draft);
    if (found) {
      setProblem(found);
      return;
    }
    if (draft.trim() !== value) onSave(draft.trim());
    setDraft(null);
  }, [draft, field, value, onSave]);

  return { draft, problem, start, cancel, change, commit };
};

export { useInlineEdit };
