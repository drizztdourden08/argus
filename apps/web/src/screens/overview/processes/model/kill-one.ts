/* @layer renderer-app @kind logic */
import { killProc } from '../../../../data/procs';
import { killedCount } from './killed-count';
import { killError } from './kill-error';
import { procLabel } from './proc-label';
import { failed } from './failed';
import { succeeded } from './succeeded';
import type { ProcItem, Said } from './model.type';

const killOne = async (proc: ProcItem, tree: boolean): Promise<Said> => {
  const label = procLabel(proc);
  try {
    const result = await killProc({ pid: Number(proc.pid), startedAt: proc.startedAt, tree });
    const count = killedCount(result, 1);
    const children = tree ? ` and its children${count > 1 ? ` (${count} processes)` : ''}` : '';
    return succeeded(`Killed ${label}${children}.`);
  } catch (error) {
    return failed(killError(error, {
      denied: `Argus will not kill ${label}: it is protected, or its rule does not allow kills.`,
      gone: `${label} is already gone, nothing was killed.`,
      changed: `${label} is not the process that was sampled (it restarted, or its pid was reused), so nothing was killed. Look again after the next sample.`,
    }));
  }
};

export { killOne };
