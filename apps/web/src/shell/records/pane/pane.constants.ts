/* @layer renderer-app @kind constants */
import { itemBody } from './item-body';
import { casesBadge } from './model/cases-badge';
import { childrenBadge } from './model/children-badge';
import { discussionBadge } from './model/discussion-badge';
import { itemBadge } from './model/item-badge';
import { linksBadge } from './model/links-badge';
import { servedBadge } from './model/served-badge';
import { testsBadge } from './model/tests-badge';
import type { PaneTab } from './pane.type';
import { placeholderBody } from './placeholder-body';
import { ActivityTab } from './tabs/ActivityTab';
import { CasesTab } from './tabs/CasesTab';
import { ChecksTab } from './tabs/ChecksTab';
import { ChildrenTab } from './tabs/ChildrenTab';
import { DiscussionTab } from './tabs/DiscussionTab';
import { FilesTab } from './tabs/FilesTab';
import { LinksTab } from './tabs/LinksTab';
import { LogTab } from './tabs/LogTab';
import { MediaTab } from './tabs/MediaTab';
import { OverviewTab } from './tabs/OverviewTab';
import { RecordActivityTab } from './tabs/RecordActivityTab';
import { RecordDiscussionTab } from './tabs/RecordDiscussionTab';
import { RecordLinksTab } from './tabs/RecordLinksTab';
import { RecordMediaTab } from './tabs/RecordMediaTab';
import { RecordOverviewTab } from './tabs/RecordOverviewTab';
import { ResultTab } from './tabs/ResultTab';
import { RunTab } from './tabs/RunTab';
import { RunsTab } from './tabs/RunsTab';
import { StepsTab } from './tabs/StepsTab';
import { TestsTab } from './tabs/TestsTab';
import { TrailTab } from './tabs/TrailTab';
import { TranscriptTab } from './tabs/TranscriptTab';
import { WorkTab } from './tabs/WorkTab';

const PANE_TABS: Readonly<Record<string, PaneTab>> = {
  overview: { label: 'Overview', icon: 'file-text', Body: itemBody(OverviewTab, RecordOverviewTab) },
  children: { label: 'Children', icon: 'git-branch', Body: itemBody(ChildrenTab, placeholderBody('children', 'Children')), badge: itemBadge(childrenBadge) },
  cases: { label: 'Cases', icon: 'list-checks', Body: itemBody(CasesTab, placeholderBody('cases', 'Cases')), badge: itemBadge(casesBadge) },
  run: { label: 'Run', icon: 'play', Body: itemBody(RunTab, placeholderBody('run', 'Run')) },
  links: { label: 'Links', icon: 'link', Body: itemBody(LinksTab, RecordLinksTab), badge: itemBadge(linksBadge) },
  discussion: { label: 'Discussion', icon: 'message-square', Body: itemBody(DiscussionTab, RecordDiscussionTab), badge: itemBadge(discussionBadge) },
  tests: { label: 'Tests', icon: 'flask', Body: itemBody(TestsTab, placeholderBody('tests', 'Tests')), badge: itemBadge(testsBadge) },
  media: { label: 'Media', icon: 'image', Body: itemBody(MediaTab, RecordMediaTab) },
  work: { label: 'Work', icon: 'package', Body: WorkTab, badge: itemBadge(servedBadge) },
  result: { label: 'Result', icon: 'circle-check', Body: ResultTab },
  runs: { label: 'Runs', icon: 'square-code', Body: RunsTab },
  log: { label: 'Log', icon: 'scroll-text', Body: LogTab },
  checks: { label: 'Checks', icon: 'list-checks', Body: ChecksTab },
  steps: { label: 'Steps', icon: 'list', Body: StepsTab },
  files: { label: 'Files', icon: 'file-text', Body: FilesTab },
  transcript: { label: 'Transcript', icon: 'message-square', Body: TranscriptTab },
  trail: { label: 'Trail', icon: 'git-branch', Body: TrailTab },
  activity: { label: 'Activity', icon: 'history', Body: itemBody(ActivityTab, RecordActivityTab) },
};

export { PANE_TABS };
