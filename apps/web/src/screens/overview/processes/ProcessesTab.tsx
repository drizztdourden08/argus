/* @layer renderer-app @kind component */
import { EmptyState, Stack } from '@drizztdourden08/tessera';
import { useKillFlow } from './behavior/useKillFlow';
import { useProcessFeed } from './behavior/useProcessFeed';
import { useProcessView } from './behavior/useProcessView';
import { NONE_LISTED, NOT_SAMPLED } from './processes.constants';
import { ProcessFilters } from './sub-components/ProcessFilters';
import { ProcessSummary } from './sub-components/ProcessSummary';
import { ProcessTable } from './sub-components/ProcessTable';
import { RuleStrip } from './sub-components/RuleStrip';
import { RulesTable } from './sub-components/RulesTable';

const ProcessesTab = () => {
  const feed = useProcessFeed();
  const view = useProcessView(feed);
  const kill = useKillFlow(feed.procs);
  const { procs, meta } = feed;
  const pick = (rule: string) => view.setFilters({ ...view.filters, rule: view.filters.rule === rule ? '' : rule });
  return (
    <Stack gap="md">
      <ProcessSummary procs={procs} meta={meta} />
      <RuleStrip groups={view.groups} rules={view.rules} procs={procs} rule={view.filters.rule} onPick={pick} kill={kill} />
      {procs.length === 0
        ? <EmptyState message={meta ? NONE_LISTED : NOT_SAMPLED} />
        : (
          <>
            <ProcessFilters filters={view.filters} onChange={view.setFilters} total={procs.length} look={view.look} rules={view.ruleNames} />
            <ProcessTable rows={view.rows} kill={kill} />
          </>
        )}
      <RulesTable rules={view.rules} />
    </Stack>
  );
};

export { ProcessesTab };
