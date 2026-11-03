/* @layer renderer-app @kind logic */
import type { OutcomeStep } from '@drizztdourden08/tessera';
import type { PlanRun } from '../run.type';
import { lastResult } from './last-result';

const caseSteps = (run: PlanRun, ids: readonly string[], titles: ReadonlyMap<string, string>, cursor: number): OutcomeStep[] =>
  ids.map((id, index) => {
    const status = lastResult(run, id)?.status;
    const outcome = index === cursor ? 'current' : (status ?? 'pending');
    const title = titles.get(id);
    return { id, outcome, label: title ? `${id} ${title}` : id };
  });

export { caseSteps };
