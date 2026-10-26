/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import type { MarkRule } from './agent-marks.type';
import { formatDuration } from './format-duration';
import { isStale } from './is-stale';
import { isStalled } from './is-stalled';
import type { AgentMark } from './model.type';

const stalledMark: MarkRule = (agent, now) => {
  if (!isStalled(agent)) return null;
  const since = Date.parse(agent.live?.lastActivityAt ?? agent.startedAt ?? agent.updatedAt ?? '');
  const quiet = Number.isFinite(since) ? ` for ${formatDuration(now - since)}` : '';
  return { id: 'stalled', label: 'stalled', tone: 'warning', tip: `No new line in the transcript${quiet}` };
};

const staleMark: MarkRule = (agent, now) =>
  isStale(agent, now) ? { id: 'stale', label: 'stale', tone: 'warning', tip: 'No update for 45 minutes or more' } : null;

const idleMark: MarkRule = (agent) =>
  agent.idleSince ? { id: 'idle', label: 'idle', tone: 'info', tip: 'The turn ended. Resume it by its agent id or close the record.' } : null;

const orphanMark: MarkRule = (agent) =>
  agent.orphanSince ? { id: 'orphan', label: 'orphan', tone: 'danger', tip: 'The session has no sub-agent or task for this record.' } : null;

const lostMark: MarkRule = (agent) =>
  agent.lost ? { id: 'lost', label: 'lost', tone: 'danger', tip: 'No task was found for it, so the record was closed.' } : null;

const duplicateMark: MarkRule = (agent) =>
  agent.duplicateOf ? { id: 'duplicate', label: `duplicate of ${agent.duplicateOf}`, tone: 'info', tip: 'Same agent id as another record.' } : null;

const idMismatchMark: MarkRule = (agent) =>
  agent.idMismatch ? { id: 'id-mismatch', label: 'id mismatch', tone: 'warning', tip: 'The recorded agent ids disagree.' } : null;

const modelMismatchMark: MarkRule = (agent) =>
  agent.modelMismatch ? { id: 'model-mismatch', label: 'model mismatch', tone: 'warning', tip: `Asked for ${agent.model ?? 'a model'}, running on ${agent.modelUsed ?? 'another'}.` } : null;

const rules: readonly MarkRule[] = [stalledMark, staleMark, idleMark, orphanMark, lostMark, duplicateMark, idMismatchMark, modelMismatchMark];

const agentMarks = (agent: Agent, now: number): AgentMark[] => rules.flatMap((rule) => rule(agent, now) ?? []);

export { agentMarks };
