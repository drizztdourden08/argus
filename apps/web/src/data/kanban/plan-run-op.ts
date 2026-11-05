/* @layer renderer-app @kind logic */
import type { DocData } from '../api';
import { sendOp } from './send-op';
import type { SentOp } from './kanban.type';
import type { PlanRunAction } from './plan-run.type';

const bodyOf = (action: PlanRunAction): DocData => {
  if (action.do === 'start') return { kind: 'plan-run-start', build: action.build };
  if (action.do === 'end') return { kind: 'plan-run-end' };
  return { kind: 'plan-run-record', test: action.test, result: action.status, note: action.note ?? '' };
};

const planRunOp = (key: string, planId: string, action: PlanRunAction): Promise<SentOp> =>
  sendOp(key, { ...bodyOf(action), target: { c: 'items', id: planId } });

export { planRunOp };
