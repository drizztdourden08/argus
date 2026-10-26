/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useCallback, useState } from 'react';
import { startRun, stopRun } from '../../../../data/runs';
import type { Card } from '../../../../data/types';
import { DOWN_SENTENCE } from '../../overview.constants';
import type { MasterRun } from '../dashboards.type';
import { runFailure } from '../model';

const useMasterRun = (card: Card): MasterRun => {
  const [busy, setBusy] = useState(false);
  const master = card.master;

  const settle = useCallback((request: Promise<string>) => {
    setBusy(true);
    request
      .then(
        (message) => toast({ variant: 'success', message }),
        (failure: unknown) => toast({ variant: 'danger', message: runFailure(failure, DOWN_SENTENCE) }),
      )
      .finally(() => setBusy(false));
  }, []);

  const play = useCallback(() => {
    const name = master?.play;
    if (busy || !name) return;
    settle(startRun(card.key, { name, visible: master.window }).then(({ run, already }) =>
      already ? `${card.name} master is already running as ${run.id}.` : `Started ${card.name} master as ${run.id}.`));
  }, [busy, card.key, card.name, master, settle]);

  const stop = useCallback(() => {
    const runId = master?.lastRun?.id;
    if (busy || !runId) return;
    settle(stopRun(card.key, runId).then(() => `Stopped ${card.name} master (${runId}).`));
  }, [busy, card.key, card.name, master, settle]);

  return { busy, play, stop };
};

export { useMasterRun };
