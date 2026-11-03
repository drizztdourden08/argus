/* @layer renderer-app @kind constants */
import { casesBadge } from './model/cases-badge';
import { childrenBadge } from './model/children-badge';
import { discussionBadge } from './model/discussion-badge';
import { linksBadge } from './model/links-badge';
import { servedBadge } from './model/served-badge';
import { testsBadge } from './model/tests-badge';
import type { PaneTab } from './pane.type';
import { ActivityTab } from './tabs/ActivityTab';
import { CasesTab } from './tabs/CasesTab';
import { ChildrenTab } from './tabs/ChildrenTab';
import { DiscussionTab } from './tabs/DiscussionTab';
import { LinksTab } from './tabs/LinksTab';
import { MediaTab } from './tabs/MediaTab';
import { OverviewTab } from './tabs/OverviewTab';
import { RunTab } from './tabs/RunTab';
import { ServedTab } from './tabs/ServedTab';
import { TestsTab } from './tabs/TestsTab';

const PANE_TABS: Readonly<Record<string, PaneTab>> = {
  overview: { label: 'Overview', icon: 'file-text', Body: OverviewTab },
  children: { label: 'Children', icon: 'git-branch', Body: ChildrenTab, badge: childrenBadge },
  cases: { label: 'Cases', icon: 'list-checks', Body: CasesTab, badge: casesBadge },
  run: { label: 'Run', icon: 'play', Body: RunTab },
  links: { label: 'Links', icon: 'link', Body: LinksTab, badge: linksBadge },
  discussion: { label: 'Discussion', icon: 'message-square', Body: DiscussionTab, badge: discussionBadge },
  tests: { label: 'Tests', icon: 'flask', Body: TestsTab, badge: testsBadge },
  media: { label: 'Media', icon: 'image', Body: MediaTab },
  builds: { label: 'Builds', icon: 'package', Body: ServedTab, badge: servedBadge },
  activity: { label: 'Activity', icon: 'history', Body: ActivityTab },
};

export { PANE_TABS };
