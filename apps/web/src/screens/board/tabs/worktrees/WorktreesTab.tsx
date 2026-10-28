/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { Panel } from '../../../../shell';
import { useAskThread } from './behavior/useAskThread';
import { useAttempt } from './behavior/useAttempt';
import { useLeftoverBatch } from './behavior/useLeftoverBatch';
import { useNotices } from './behavior/useNotices';
import { usePanel } from './behavior/usePanel';
import { useRemoval } from './behavior/useRemoval';
import { useTabPrefs } from './behavior/useTabPrefs';
import { useWorktreesData } from './behavior/useWorktreesData';
import { countRows, filterRows, refusedJobs } from './model';
import { LeftoverBatchConfirm } from './sub-components/LeftoverBatchConfirm';
import { NoticeBanner } from './sub-components/NoticeBanner';
import { RefusedJobs } from './sub-components/RefusedJobs';
import { RequestsSection } from './sub-components/RequestsSection';
import { RulesSection } from './sub-components/RulesSection';
import { WorktreeBody } from './sub-components/WorktreeBody';
import { WorktreeSummary } from './sub-components/WorktreeSummary';
import { WorktreeToolbar } from './sub-components/WorktreeToolbar';
import type { WorktreesTabProps } from './worktrees-props.type';

const WorktreesTab = (props: WorktreesTabProps) => {
  const { boardKey } = props;
  const { ready, meta, rows } = useWorktreesData(boardKey);
  const prefs = useTabPrefs();
  const notices = useNotices();
  const controls = usePanel(meta);
  const guard = useAttempt();
  const ask = useAskThread(boardKey, meta, controls, guard);
  const removal = useRemoval(boardKey, controls, guard, notices);
  const batch = useLeftoverBatch(boardKey, meta, guard, notices);
  if (!ready) return <Text tone="muted">Loading the worktrees.</Text>;
  const counts = countRows(rows);
  const rules = meta?.rules ?? [];
  return (
    <Panel
      title="Worktrees"
      note={rules.length > 0 ? <WorktreeSummary counts={counts} meta={meta} /> : undefined}
      tools={<WorktreeToolbar filter={prefs.filter} onFilter={prefs.setFilter} counts={counts} meta={meta} batch={batch} />}
    >
      <LeftoverBatchConfirm meta={meta} batch={batch} />
      {notices.notice && <NoticeBanner notice={notices.notice} onDismiss={notices.clear} />}
      <RefusedJobs jobs={refusedJobs(meta, notices.dismissed, Date.now())} onDismiss={notices.dismissJobs} />
      <WorktreeBody
        rows={filterRows(rows, prefs.filter)}
        total={counts.all}
        filter={prefs.filter}
        hasRules={rules.length > 0}
        meta={meta}
        controls={controls}
        ask={ask}
        removal={removal}
        onShowAll={() => prefs.setFilter('all')}
      />
      <RequestsSection requests={meta?.requests ?? []} />
      <RulesSection rules={rules} meta={meta} open={prefs.rulesOpen} onOpen={prefs.setRulesOpen} />
    </Panel>
  );
};

export { WorktreesTab };
