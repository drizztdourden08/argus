/* @layer renderer-app @kind logic */
import { useCallback, useMemo, useReducer, useState } from 'react';
import type { RunStatus } from '../../../../../data/kanban';
import { caseSteps } from '../model/case-steps';
import { firstOpen } from '../model/first-open';
import { noteProblem } from '../model/note-problem';
import { runnerReducer } from '../model/runner-reducer';
import type { RunStageProps } from '../run.type';

const useRunStage = (props: RunStageProps) => {
  const { data, run, ids, ops } = props;
  const total = ids.length;
  const [state, dispatch] = useReducer(runnerReducer, undefined, () => ({ cursor: firstOpen(run, ids), note: '', asked: null }));
  const [busy, setBusy] = useState(false);
  const id = ids[state.cursor] ?? '';
  const test = data.allTests.find((candidate) => candidate.id === id);
  const titles = useMemo(() => new Map(data.allTests.map((candidate) => [candidate.id, candidate.title])), [data.allTests]);
  const steps = useMemo(() => caseSteps(run, ids, titles, state.cursor), [run, ids, titles, state.cursor]);
  const send = useCallback(
    async (status: RunStatus) => {
      if (busy || !test) return;
      if (noteProblem(status, state.note)) {
        dispatch({ type: 'ask', status });
        return;
      }
      setBusy(true);
      const ok = await ops.record(id, status, state.note.trim());
      setBusy(false);
      if (ok) dispatch({ type: 'recorded', total });
    },
    [busy, test, state.note, ops, id, total],
  );
  const record = useCallback(
    (status: RunStatus) => {
      send(status).catch(() => setBusy(false));
    },
    [send],
  );
  const step = useCallback((by: 1 | -1) => dispatch({ type: 'step', by, total }), [total]);
  const goto = useCallback((index: number) => dispatch({ type: 'goto', index, total }), [total]);
  const setNote = useCallback((text: string) => dispatch({ type: 'note', text }), []);
  const problem = state.asked ? noteProblem(state.asked, state.note) : null;
  return { id, test, steps, cursor: state.cursor, note: state.note, problem, busy, record, step, goto, setNote };
};

export { useRunStage };
