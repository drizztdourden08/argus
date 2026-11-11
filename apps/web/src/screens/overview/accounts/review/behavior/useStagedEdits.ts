/* @layer renderer-app @kind logic */
import { useCallback, useRef, useState } from 'react';
import type { ReviewEdit } from '../../../../../data/types';
import { editSlot } from '../model';
import type { EditStage, StagedEdit } from '../review.type';

const useStagedEdits = (): EditStage => {
  const [items, setItems] = useState<readonly StagedEdit[]>([]);
  const counter = useRef(0);

  const add = useCallback((edits: readonly ReviewEdit[]) => {
    const fresh = edits.map((edit): StagedEdit => ({ key: ++counter.current, edit }));
    setItems((prev) => {
      const slots = new Set(edits.map(editSlot));
      return [...prev.filter((item) => !slots.has(editSlot(item.edit))), ...fresh];
    });
  }, []);

  const remove = useCallback((key: number) => setItems((prev) => prev.filter((item) => item.key !== key)), []);

  return { items, add, remove };
};

export { useStagedEdits };
