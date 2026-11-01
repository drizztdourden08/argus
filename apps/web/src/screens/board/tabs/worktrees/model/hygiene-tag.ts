/* @layer renderer-app @kind logic */
import type { WorktreeHygiene } from '../../../../../data/worktrees';
import { plainText } from '../../../plain-text';
import type { HygieneTag } from './hygiene-tag.type';
import { HYGIENE_LOOK, UNKNOWN_STATUS_LOOK } from './model.constants';

const clockText = (iso: string): string => new Date(iso).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });

const releasedTag = (hygiene: WorktreeHygiene): Omit<HygieneTag, 'look'> => {
  const thread = hygiene.thread ?? 'the thread';
  const when = hygiene.goesAt ? `at ${clockText(hygiene.goesAt)}` : `one grace after ${thread} is told`;
  return {
    text: hygiene.goesAt ? `released, goes ${clockText(hygiene.goesAt)}` : 'released',
    tip: `No running agent owns it, nothing uses it and removing it loses nothing: Argus removes it ${when}, unless ${thread} keeps it (coord worktree keep) or releases it now (coord worktree release).`,
  };
};

const hygieneTag = (hygiene: WorktreeHygiene): HygieneTag => {
  const look = HYGIENE_LOOK[hygiene.state] ?? UNKNOWN_STATUS_LOOK;
  const why = hygiene.why ? plainText(hygiene.why) : '';
  if (hygiene.state === 'released') return { ...releasedTag(hygiene), look };
  if (hygiene.state === 'in-use') return { text: 'in use', tip: `In use, so Argus leaves it${why ? `: ${why}` : ''}.`, look };
  if (hygiene.state === 'kept') return { text: 'kept', tip: `The thread keeps it${why ? `: ${why}` : ''}.`, look };
  return { text: 'review', tip: `Argus does not remove it by itself: ${why || 'the thread decides'}.`, look };
};

export { hygieneTag };
