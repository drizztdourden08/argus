/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useCallback, useState } from 'react';
import { askStillValid, killOne, killStale, staleOf } from '../model';
import type { KillAsk, ProcItem, Said } from '../model';
import { GONE } from '../processes.constants';
import type { KillFlow } from '../processes.type';

const useKillFlow = (procs: readonly ProcItem[]): KillFlow => {
  const [asked, setAsked] = useState<KillAsk | null>(null);
  const [killing, setKilling] = useState(false);
  const ask = askStillValid(asked, procs);

  const run = useCallback((action: () => Promise<Said>) => {
    setAsked(null);
    setKilling(true);
    void action()
      .then((said) => toast({ variant: said.variant, message: said.message }))
      .finally(() => setKilling(false));
  }, []);

  const killProcess = useCallback((id: string, tree: boolean) => {
    const proc = procs.find((item) => item.id === id);
    if (killing) return;
    if (!proc) {
      setAsked(null);
      toast({ variant: 'danger', message: GONE });
      return;
    }
    run(() => killOne(proc, tree));
  }, [killing, procs, run]);

  const killStaleOf = useCallback((rule: string) => {
    if (killing) return;
    run(() => killStale(rule, staleOf(procs, rule).length));
  }, [killing, procs, run]);

  return {
    ask,
    killing,
    askProcess: (id) => setAsked({ kind: 'proc', id }),
    askRule: (rule) => setAsked({ kind: 'rule', rule }),
    cancel: () => setAsked(null),
    killProcess,
    killStale: killStaleOf,
  };
};

export { useKillFlow };
