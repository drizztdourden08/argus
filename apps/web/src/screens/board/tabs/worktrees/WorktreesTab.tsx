/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { TabPanel } from '../../shared';
import { useAskThread } from './behavior/useAskThread';
import { useAttempt } from './behavior/useAttempt';
import { useLeftoverBatch } from './behavior/useLeftoverBatch';
import { useNotices } from './behavior/useNotices';
import { usePanel } from './behavior/usePanel';
import { useRemoval } from './behavior/useRemoval';
import { useTabPrefs } from './behavior/useTabPrefs';
import { useWorktreesData } from './behavior/useWorktreesData';
import { applyRowFilters, countRows, filterRows, refusedJobs, removedOf, requestsOf, sortRows, worktreeStats } from './model';
import { LeftoverBatchConfirm } from './sub-components/LeftoverBatchConfirm';
import { NoticeBanner } from './sub-components/NoticeBanner';
import { RefusedJobs } from './sub-components/RefusedJobs';
import { RemovedSection } from './sub-components/RemovedSection';
import { RequestsSection } from './sub-components/RequestsSection';
import { WorktreeBody } from './sub-components/WorktreeBody';
import { WorktreeScanned } from './sub-components/WorktreeScanned';
import { WorktreeToolbar } from './sub-components/WorktreeToolbar';
import type { WorktreesTabProps } from './worktrees-props.type';

const WorktreesTab = (props: WorktreesTabProps) => {
  const { boardKey } = props;
  const { ready, meta, rows } = useWorktreesData(boardKey);
  const prefs = useTabPrefs(boardKey);
  const notices = useNotices();
  const controls = usePanel(meta);
  const guard = useAttempt();
  const ask = useAskThread(boardKey, meta, controls, guard);
  const removal = useRemoval(boardKey, controls, guard, notices);
  const batch = useLeftoverBatch(boardKey, meta, guard, notices);
  if (!ready) return <Text tone="muted">Loading the worktrees.</Text>;
  const counts = countRows(rows);
  const rules = meta?.rules ?? [];
  const { view } = prefs;
  const shown = sortRows(applyRowFilters(filterRows(rows, prefs.needOnly), view.filters, meta), view.order, view.direction);
  return (
    <TabPanel label="Worktrees" stats={worktreeStats(counts, meta)} tools={rules.length > 0 ? <WorktreeScanned scannedAt={meta?.scannedAt} /> : undefined}>
      <WorktreeToolbar boardKey={boardKey} rows={rows} prefs={prefs} counts={counts} meta={meta} batch={batch} />
      <LeftoverBatchConfirm meta={meta} batch={batch} />
      {notices.notice && <NoticeBanner notice={notices.notice} onDismiss={notices.clear} />}
      <RefusedJobs jobs={refusedJobs(meta, notices.dismissed, Date.now())} onDismiss={notices.dismissJobs} />
      <WorktreeBody
        boardKey={boardKey}
        rows={shown}
        group={view.group}
        layout={prefs.layout}
        compact={prefs.compact}
        total={counts.all}
        needOnly={prefs.needOnly}
        hasRules={rules.length > 0}
        meta={meta}
        controls={controls}
        ask={ask}
        removal={removal}
        onShowAll={() => prefs.setNeedOnly(false)}
      />
      <RequestsSection boardKey={boardKey} requests={requestsOf(meta)} />
      <RemovedSection boardKey={boardKey} removed={removedOf(meta)} />
    </TabPanel>
  );
};

export { WorktreesTab };
