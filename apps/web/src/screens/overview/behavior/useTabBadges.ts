/* @layer renderer-app @kind logic */
import type { TabBadgeTone } from '@drizztdourden08/tessera';
import { useMemo } from 'react';
import { useDeps } from '../../../data/deps';
import { PORTFOLIO_SCOPE, useCollection } from '../../../data/react';
import type { ActivityDoc, NotificationDoc, ProcDoc } from '../../../data/types';
import { levelOf, needsAttention } from '../activity/model';
import { depBadge } from '../deps/model';
import { noteLevel } from '../notifications/model';
import { needsLook } from '../processes/model';
import { useUnread } from '../seen';
import { ACTIVITY_SEEN_KEY, NOTIFICATIONS_SEEN_KEY } from '../overview.constants';
import type { TabBadges } from '../overview.type';

const toneOf = (count: number, tone: TabBadgeTone): TabBadgeTone => (count === 0 ? 'neutral' : tone);

const useTabBadges = (tab: string): TabBadges => {
  const procs = useCollection<ProcDoc>(PORTFOLIO_SCOPE, 'procs');
  const activity = useCollection<ActivityDoc>(PORTFOLIO_SCOPE, 'activity');
  const notes = useCollection<NotificationDoc>(PORTFOLIO_SCOPE, 'notifications');
  const deps = useDeps();
  const waiting = useMemo(() => depBadge(deps.docs, Date.now()), [deps.docs]);
  const look = procs.docs.filter(needsLook).length;
  const attention = useMemo(() => activity.docs.filter(needsAttention), [activity.docs]);
  const unreadActivity = useUnread(ACTIVITY_SEEN_KEY, attention, tab === 'activity');
  const unreadNotes = useUnread(NOTIFICATIONS_SEEN_KEY, notes.docs, tab === 'notifications');
  return {
    counts: {
      processes: look,
      dependencies: waiting,
      activity: unreadActivity.unread.length,
      notifications: unreadNotes.unread.length,
    },
    tones: {
      processes: toneOf(look, 'warn'),
      dependencies: toneOf(waiting, 'warn'),
      activity: toneOf(unreadActivity.unread.length, unreadActivity.unread.some((item) => levelOf(item) === 'error') ? 'danger' : 'warn'),
      notifications: toneOf(unreadNotes.unread.length, unreadNotes.unread.some((note) => noteLevel(note) === 'urgent') ? 'danger' : 'warn'),
    },
    freshActivity: unreadActivity.freshSince,
    freshNotifications: unreadNotes.freshSince,
  };
};

export { useTabBadges };
