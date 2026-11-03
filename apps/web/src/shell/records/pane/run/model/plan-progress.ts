/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import { caseIds } from '../../model/case-ids';
import { EMPTY_SUMMARY } from '../run.constants';
import type { PlanProgress } from '../run.type';
import { runsOf } from './runs-of';
import { shownRun } from './shown-run';
import { summarize } from './summarize';

const planProgress = (item: Pick<ItemRef, 'fields'>): PlanProgress => {
  const ids = caseIds(item);
  const run = shownRun(runsOf(item));
  if (!run) return { summary: { ...EMPTY_SUMMARY, total: ids.length, notRun: ids.length }, live: false, cases: ids.length };
  return { summary: summarize(run, ids), live: run.endedAt === null, cases: ids.length };
};

export { planProgress };
