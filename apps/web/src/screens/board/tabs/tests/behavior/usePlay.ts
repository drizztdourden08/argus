/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { startRun, stopRun } from '../../../../../data/runs';
import { playsInWindow } from '../model';
import type { LaunchDefDoc, RunEntry } from '../model';
import type { Play } from '../tests.type';
import { useRunAction } from './useRunAction';

const usePlay = (boardKey: string): Play => {
  const { busy, act } = useRunAction();
  const start = useCallback((def: LaunchDefDoc, tag: string) =>
    act(async () => {
      const { run } = await startRun(boardKey, { name: def.name, visible: playsInWindow(def) });
      return `${tag} started as ${run.id}.`;
    }), [act, boardKey]);
  const stop = useCallback((run: RunEntry) =>
    act(async () => {
      await stopRun(boardKey, run.id);
      return `${run.id} stopped.`;
    }), [act, boardKey]);
  return { busy, start, stop };
};

export { usePlay };
