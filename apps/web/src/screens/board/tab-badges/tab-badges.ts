/* @layer renderer-app @kind logic */
import type { TabBadgeTone } from '@drizztdourden08/tessera';
import type { Agent } from '../../../data/agents';
import type { Launcher } from '../../../data/types';
import type { Worktree } from '../../../data/worktrees';
import type { BoardWork } from '../feed-model';
import { ACTIVE_AGENT_STATUSES, NEEDS_DECISION_STATUSES, STALE_AGENT_MS } from './tab-badges.constants';
import type { Badge, BoardBadges } from './tab-badges.type';

const isLive = (doc: { archivedAt?: string | null; deletedAt?: string | null }): boolean => !doc.archivedAt && !doc.deletedAt;

const isQuiet = (agent: Agent, now: number): boolean =>
  agent.live?.stalled === true || (Boolean(agent.updatedAt) && now - Date.parse(agent.updatedAt ?? '') > STALE_AGENT_MS);

const toneOf = (on: boolean, tone: TabBadgeTone): TabBadgeTone | undefined => (on ? tone : undefined);

const agentsBadge = (agents: readonly Agent[], now: number): Badge => {
  const active = agents.filter((agent) => isLive(agent) && ACTIVE_AGENT_STATUSES.includes(agent.status));
  return { count: active.length, tone: toneOf(active.some((agent) => agent.status === 'working' && isQuiet(agent, now)), 'warn') };
};

const activityBadge = (work: BoardWork): Badge => {
  const cards = [...work.waiting, ...work.notices];
  const tone = work.waiting.some((card) => card.type === 'decision') ? 'danger' : toneOf(work.waiting.length > 0, 'warn');
  return { count: cards.length, tone };
};

const needsYou = (work: BoardWork, id: string): boolean => {
  const needs = work.needs.get(id);
  return Boolean(needs && needs.questions + needs.decisions + needs.steps + needs.tests > 0);
};

const kanbanBadge = (work: BoardWork): Badge => ({
  count: work.items.length,
  tone: toneOf(work.items.some((item) => needsYou(work, item.id)), 'warn'),
});

const buildsBadge = (launchers: readonly Launcher[]): Badge => ({ count: launchers.filter((launcher) => !launcher.retiredAt).length });

const worktreesBadge = (worktrees: readonly Worktree[]): Badge => {
  const waiting = worktrees.filter((worktree) => NEEDS_DECISION_STATUSES.includes(worktree.status));
  return { count: waiting.length, tone: toneOf(waiting.length > 0, 'warn') };
};

const testsBadge = (count: number): Badge => ({ count, tone: toneOf(count > 0, 'warn') });

const boardBadges = (input: {
  work: BoardWork;
  agents: readonly Agent[];
  launchers: readonly Launcher[];
  worktrees: readonly Worktree[];
  toRun: number;
  now: number;
}): BoardBadges => ({
  agents: agentsBadge(input.agents, input.now),
  activity: activityBadge(input.work),
  kanban: kanbanBadge(input.work),
  tests: testsBadge(input.toRun),
  builds: buildsBadge(input.launchers),
  worktrees: worktreesBadge(input.worktrees),
});

export { boardBadges };
