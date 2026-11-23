/* @layer renderer-app @kind logic */
import { useCallback, useState } from 'react';
import { writeErrorText } from '../../../../../data/api';
import { sendOp } from '../../../../../data/kanban';
import { useDoc } from '../../../../../data/react';
import type { KanbanConfig } from '../../../../../data/types';
import { postVerdict } from '../../tests/model';
import { DEFAULT_CONFIG, configIsDefault } from '../model';
import type { ConfigEditor } from '../config-editor.type';

const useConfigEditor = (boardKey: string): ConfigEditor => {
  const stored = useDoc<KanbanConfig>(boardKey, 'meta', 'kanban').doc ?? DEFAULT_CONFIG;
  const [draft, setDraft] = useState<KanbanConfig | null>(null);
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);
  const send = useCallback(async (kanban: KanbanConfig | null) => {
    setBusy(true);
    try {
      const sent = await sendOp(boardKey, { kind: 'kanban-config', kanban });
      const verdict = postVerdict(sent.result);
      setProblem(verdict);
      if (!verdict) setDraft(null);
    } catch (error) {
      setProblem(writeErrorText(error));
    } finally {
      setBusy(false);
    }
  }, [boardKey]);
  const config = draft ?? stored;
  return {
    config,
    custom: !configIsDefault(stored),
    dirty: draft !== null,
    busy,
    problem,
    edit: (change) => setDraft(change(config)),
    save: () => send(config),
    useDefault: () => send(null),
    discard: () => {
      setDraft(null);
      setProblem(null);
    },
  };
};

export { useConfigEditor };
