/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useCallback, useState } from 'react';
import { sendDown } from '../../../../data/posts';
import { useProjectName } from '../../projects';
import type { SendDown } from '../dashboards.type';
import { sendFailure, sendOutcome, sendProblem } from '../model';
import type { SendKind } from '../model';

const useSendDown = (): SendDown => {
  const nameOf = useProjectName();
  const [target, setTarget] = useState<string | null>(null);
  const [kind, setKind] = useState<SendKind>('decision');
  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const [busy, setBusy] = useState(false);
  const text = target ? (drafts[target] ?? '') : '';
  const name = target === 'all' ? 'all projects' : nameOf(target ?? '');

  const setText = useCallback((value: string) => {
    if (target) setDrafts((current) => ({ ...current, [target]: value }));
  }, [target]);

  const send = useCallback(() => {
    if (!target || busy) return;
    const problem = sendProblem(text.trim());
    if (problem) {
      toast({ variant: 'danger', message: problem });
      return;
    }
    setBusy(true);
    sendDown({ project: target, text: text.trim(), decision: kind === 'decision' })
      .then(
        (result) => {
          const outcome = sendOutcome(result, name);
          toast({ variant: outcome.ok ? 'success' : 'danger', message: outcome.message });
          if (outcome.ok) {
            setDrafts((current) => ({ ...current, [target]: '' }));
            setTarget(null);
          }
        },
        (failure: unknown) => toast({ variant: 'danger', message: sendFailure(failure) }),
      )
      .finally(() => setBusy(false));
  }, [busy, kind, name, target, text]);

  return { target, name, kind, text, busy, open: setTarget, close: () => setTarget(null), setKind, setText, send };
};

export { useSendDown };
