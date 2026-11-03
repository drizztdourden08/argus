/* @layer renderer-app @kind logic */
import { planProgress } from '../../../../../shell';
import { PLAN_CLOSED_STAGES } from './model.constants';
import type { Counts, ItemDoc, PlanView } from './model.type';
import { casesOf } from './plan-cases';

const countsOf = (item: ItemDoc): Pick<PlanView, 'counts' | 'live'> => {
  const { summary, live } = planProgress(item);
  const counts: Counts = { run: summary.notRun + summary.skip, pass: summary.pass, fail: summary.fail, partial: summary.partial, blocked: summary.blocked };
  return { counts, live };
};

const planViews = (items: readonly ItemDoc[], only: string): PlanView[] =>
  items
    .filter((item) => item.type === 'test-plan' && !item.deletedAt && (only === '' || item.id === only))
    .sort((a, b) => a.id.localeCompare(b.id, undefined, { numeric: true }))
    .map((item) => ({ item, cases: casesOf(item), closed: PLAN_CLOSED_STAGES.includes(item.stage ?? ''), ...countsOf(item) }));

export { planViews };
