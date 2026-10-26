/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { playsInWindow, startRun, stopRun } from '../../../../data/runs';
import type { RunEntry } from '../../../../data/runs';
import type { Play, RunDef } from '../run-play.type';
import { useRunAction } from './useRunAction';

const usePlay = (boardKey: string): Play => {
  const { busy, act } = useRunAction();
  const start = useCallback((def: RunDef, label: string) =>
    act(async () => {
      const { run } = await startRun(boardKey, { name: def.name, visible: playsInWindow(def) });
      return `${label} started as ${run.id}.`;
    }), [act, boardKey]);
  const stop = useCallback((run: RunEntry) =>
    act(async () => {
      await stopRun(boardKey, run.id);
      return `${run.id} stopped.`;
    }), [act, boardKey]);
  return { busy, start, stop };
};

export { usePlay };
