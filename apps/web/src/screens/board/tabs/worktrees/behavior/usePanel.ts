/* @layer renderer-app @kind logic */
import { useCallback, useMemo, useState } from 'react';
import type { WorktreesMeta } from '../../../../../data/worktrees';
import { nextPanel } from '../model';
import type { Panel, PanelKind, WorktreeRow } from '../model';
import type { PanelControls } from '../worktrees.type';

const usePanel = (meta: WorktreesMeta | null): PanelControls => {
  const [panel, setPanel] = useState<Panel | null>(null);
  const toggle = useCallback((row: WorktreeRow, kind: PanelKind) => setPanel((current) => nextPanel(current, row, kind, meta)), [meta]);
  const close = useCallback(() => setPanel(null), []);
  const patch = useCallback((change: Partial<Panel>) => setPanel((current) => (current ? { ...current, ...change } : current)), []);
  return useMemo(() => ({ panel, toggle, close, patch }), [panel, toggle, close, patch]);
};

export { usePanel };
