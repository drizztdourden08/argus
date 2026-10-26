/* @layer renderer-app @kind logic */
import { killRule } from '../../../../data/procs';
import { killedCount } from './killed-count';
import { killError } from './kill-error';
import { failed } from './failed';
import { succeeded } from './succeeded';
import type { Said } from './model.type';

const killStale = async (rule: string, expected: number): Promise<Said> => {
  try {
    const count = killedCount(await killRule(rule), expected);
    return succeeded(`Killed ${count} stale ${rule} process${count === 1 ? '' : 'es'}.`);
  } catch (error) {
    return failed(killError(error, {
      denied: `Argus will not kill the ${rule} processes: the rule does not allow kills.`,
      gone: `No stale ${rule} process is left, nothing was killed.`,
      changed: `The ${rule} processes changed since they were sampled, so nothing was killed. Look again after the next sample.`,
    }));
  }
};

export { killStale };
