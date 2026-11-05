/* @layer renderer-app @kind logic */
import { useState } from 'react';
import { changedFields, editProblem } from '../model';
import type { EditChange, TestDoc } from '../model';
import type { EditDraft, TestOps } from '../tests.type';

const startValue = (test: TestDoc): EditChange => ({ title: test.title, covers: test.covers ?? '', mode: test.mode });

const useEditDraft = (test: TestDoc, ops: TestOps, onDone: () => void): EditDraft => {
  const [value, setValue] = useState<EditChange>(() => startValue(test));
  const [item, setItem] = useState(test.item ?? '');
  const [problem, setProblem] = useState<string | null>(null);
  const set = (field: keyof EditChange, next: string) => {
    setProblem(null);
    setValue((now) => ({ ...now, [field]: next }));
  };
  const save = () => {
    const found = editProblem(value);
    setProblem(found);
    if (found) return;
    const changed = changedFields(test, value);
    if (Object.keys(changed).length > 0) void ops.edit(test, changed);
    if (item && item !== test.item) void ops.move(test, item);
    onDone();
  };
  return { value, item, problem, set, setItem, save };
};

export { useEditDraft };
