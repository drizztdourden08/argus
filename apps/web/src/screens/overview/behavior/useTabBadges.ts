/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { PORTFOLIO_SCOPE, useCollection } from '../../../data/react';
import type { ActivityDoc, NotificationDoc, ProcDoc } from '../../../data/types';
import { needsAttention } from '../activity/model';
import { needsLook } from '../processes/model';
import { useUnread } from '../seen';
import { ACTIVITY_SEEN_KEY, NOTIFICATIONS_SEEN_KEY } from '../overview.constants';
import type { TabBadges } from '../overview.type';

const useTabBadges = (tab: string): TabBadges => {
  const procs = useCollection<ProcDoc>(PORTFOLIO_SCOPE, 'procs');
  const activity = useCollection<ActivityDoc>(PORTFOLIO_SCOPE, 'activity');
  const notes = useCollection<NotificationDoc>(PORTFOLIO_SCOPE, 'notifications');
  const attention = useMemo(() => activity.docs.filter(needsAttention), [activity.docs]);
  const unreadActivity = useUnread(ACTIVITY_SEEN_KEY, attention, tab === 'activity');
  const unreadNotes = useUnread(NOTIFICATIONS_SEEN_KEY, notes.docs, tab === 'notifications');
  return {
    counts: {
      dashboards: 0,
      processes: procs.docs.filter(needsLook).length,
      activity: unreadActivity.unread.length,
      notifications: unreadNotes.unread.length,
    },
    freshActivity: unreadActivity.freshSince,
    freshNotifications: unreadNotes.freshSince,
  };
};

export { useTabBadges };
