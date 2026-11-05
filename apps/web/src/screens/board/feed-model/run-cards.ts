/* @layer renderer-app @kind logic */
import type { WorkItem } from '../../../data/types';
import { OWNER_ASSIGNEE } from './feed.constants';
import type { FeedCard, RunCard } from './feed.type';

const textOf = (value: unknown): string => (typeof value === 'string' ? value : '');

const casesOf = (plan: WorkItem | undefined): number => {
  const cases = plan?.fields?.cases;
  return Array.isArray(cases) ? cases.length : 0;
};

const isWaitingRun = (item: WorkItem): boolean => {
  if (item.type !== 'test-run' || item.deletedAt || item.archivedAt) return false;
  return item.stage === 'reported' || (item.stage === 'ready' && item.assignee === OWNER_ASSIGNEE);
};

const runCard = (item: WorkItem, plans: ReadonlyMap<string, WorkItem>): FeedCard => {
  const plan = textOf(item.fields?.plan);
  const run: RunCard = {
    id: item.id,
    n: item.n ?? 0,
    plan,
    planTitle: plans.get(plan)?.title ?? plan,
    build: textOf(item.fields?.build) || null,
    cases: casesOf(plans.get(plan)),
    thread: item.thread,
    reported: item.stage === 'reported',
  };
  return { type: 'run', at: item.updatedAt ?? '', data: run };
};

const runCards = (items: WorkItem[]): FeedCard[] => {
  const plans = new Map(items.map((item) => [item.id, item]));
  return items.filter(isWaitingRun).map((item) => runCard(item, plans));
};

export { runCards };
