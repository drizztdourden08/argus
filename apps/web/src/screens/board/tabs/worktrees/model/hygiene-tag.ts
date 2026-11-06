/* @layer renderer-app @kind logic */
import type { WorktreeHygiene } from '../../../../../data/worktrees';
import { plainText } from '../../../plain-text';
import type { HygieneTag } from './hygiene-tag.type';
import { HYGIENE_LOOK, UNKNOWN_STATUS_LOOK } from './model.constants';

const clockText = (iso: string): string => new Date(iso).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });

const removingTag = (hygiene: WorktreeHygiene): Omit<HygieneTag, 'look'> => {
  const when = hygiene.goesAt ? `at ${clockText(hygiene.goesAt)}` : 'now';
  return {
    text: hygiene.goesAt ? `removing, goes ${clockText(hygiene.goesAt)}` : 'removing',
    tip: `Nothing is attached to it and removing it loses nothing: Argus removes it ${when}, its merged branch too. An agent, a run or a process on it stops that.`,
  };
};

const awaitingTag = (hygiene: WorktreeHygiene, why: string): Omit<HygieneTag, 'look'> => {
  const lead = hygiene.thread ?? 'its lead';
  const asked = hygiene.askedAt ? `${lead} was asked at ${clockText(hygiene.askedAt)}` : `${lead} is asked`;
  return {
    text: hygiene.askedAt ? 'lead asked' : 'awaiting lead',
    tip: `Nothing is attached to it but it holds work: ${why || 'work would be lost'}. ${asked} to approve the removal (coord worktree approve) or to ask the Overlord for an exception. Unanswered, it goes to the Overlord.`,
  };
};

const reportedTag = (hygiene: WorktreeHygiene, why: string): Omit<HygieneTag, 'look'> => {
  const at = hygiene.reportedAt ? ` at ${clockText(hygiene.reportedAt)}` : '';
  return {
    text: 'with the Overlord',
    tip: `Its lead did not answer: the Overlord was told${at} to delete it or keep it. Left on it: ${why || 'work'}.`,
  };
};

const exceptionTag = (hygiene: WorktreeHygiene, why: string): Omit<HygieneTag, 'look'> => {
  const until = hygiene.until ? ` until ${hygiene.until.slice(0, 10)}` : '';
  return {
    text: 'exception',
    tip: `On the Overlord's exception list${until}${why ? `: ${why}` : ''}. Argus never removes or reports it.`,
    edge: 'dashed',
  };
};

const shownOf = (hygiene: WorktreeHygiene, why: string): Omit<HygieneTag, 'look'> => {
  if (hygiene.state === 'removing') return removingTag(hygiene);
  if (hygiene.state === 'awaiting-lead') return awaitingTag(hygiene, why);
  if (hygiene.state === 'reported') return reportedTag(hygiene, why);
  if (hygiene.state === 'exception') return exceptionTag(hygiene, why);
  return { text: 'attached', tip: `Attached, so Argus leaves it${why ? `: ${why}` : ''}.` };
};

const hygieneTag = (hygiene: WorktreeHygiene): HygieneTag => {
  const look = HYGIENE_LOOK[hygiene.state] ?? UNKNOWN_STATUS_LOOK;
  const why = hygiene.why ? plainText(hygiene.why) : '';
  return { ...shownOf(hygiene, why), look };
};

export { hygieneTag };
