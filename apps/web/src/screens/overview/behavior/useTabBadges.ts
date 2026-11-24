/* @layer renderer-app @kind logic */
import type { TabBadgeTone } from '@drizztdourden08/tessera';
import { useMemo } from 'react';
import { PORTFOLIO_SCOPE, useCollection } from '../../../data/react';
import type { AccountDoc, ActivityDoc, ProcDoc } from '../../../data/types';
import { countCritical } from '../accounts';
import { levelOf, needsAttention } from '../activity/model';
import { needsLook } from '../processes/model';
import { useUnread } from '../seen';
import { ACTIVITY_SEEN_KEY } from '../overview.constants';
import type { TabBadges } from '../overview.type';

const toneOf = (count: number, tone: TabBadgeTone): TabBadgeTone => (count === 0 ? 'neutral' : tone);

const useTabBadges = (tab: string): TabBadges => {
  const procs = useCollection<ProcDoc>(PORTFOLIO_SCOPE, 'procs');
  const accounts = useCollection<AccountDoc>(PORTFOLIO_SCOPE, 'accounts');
  const activity = useCollection<ActivityDoc>(PORTFOLIO_SCOPE, 'activity');
  const look = procs.docs.filter(needsLook).length;
  const critical = countCritical(accounts.docs);
  const attention = useMemo(() => activity.docs.filter(needsAttention), [activity.docs]);
  const unreadActivity = useUnread(ACTIVITY_SEEN_KEY, attention, tab === 'activity');
  return {
    counts: {
      processes: look,
      accounts: critical,
      activity: unreadActivity.unread.length,
    },
    tones: {
      processes: toneOf(look, 'warn'),
      accounts: toneOf(critical, 'danger'),
      activity: toneOf(unreadActivity.unread.length, unreadActivity.unread.some((item) => levelOf(item) === 'error') ? 'danger' : 'warn'),
    },
    freshActivity: unreadActivity.freshSince,
  };
};

export { useTabBadges };
