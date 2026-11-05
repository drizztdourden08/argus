/* @layer renderer-app @kind logic */
import { planCode } from './plan-code';
import type { TestDoc, TestGroup, TestsView } from './model.type';

const idsOf = (tests: readonly TestDoc[]): string[] => tests.map((test) => test.id);

const casesShown = (view: TestsView, group: TestGroup): string[] => {
  if (group === 'run') return view.runs.flatMap((run) => idsOf(run.tests));
  if (group !== 'plan') return view.cards.flatMap((card) => idsOf(card.tests));
  const { groups, none, closed } = view.byPlan;
  return [...groups, ...closed].flatMap((entry) => idsOf(entry.rows)).concat(idsOf(none));
};

const shownIds = (view: TestsView, group: TestGroup): string[] => {
  const plans = group === 'plan' ? view.plans.map((plan) => planCode(plan.item.id)) : [];
  return [...new Set([...plans, ...casesShown(view, group)])];
};

export { shownIds };
