/* @layer renderer-app @kind component */
import { EmptyState, Stack } from '@drizztdourden08/tessera';
import { useNow } from '../clock';
import { useKillFlow } from './behavior/useKillFlow';
import { useProcessFeed } from './behavior/useProcessFeed';
import { useProcessView } from './behavior/useProcessView';
import { NONE_LISTED, NOT_SAMPLED, SAMPLE_TICK_MS } from './processes.constants';
import { ProcessDetail } from './sub-components/ProcessDetail';
import { ProcessFilters } from './sub-components/ProcessFilters';
import { ProcessSummary } from './sub-components/ProcessSummary';
import { ProcessTable } from './sub-components/ProcessTable';
import { RuleStrip } from './sub-components/RuleStrip';
import { RulesTable } from './sub-components/RulesTable';

const ProcessesTab = () => {
  const feed = useProcessFeed();
  const view = useProcessView(feed);
  const kill = useKillFlow(feed.procs);
  const now = useNow(SAMPLE_TICK_MS);
  const { procs, meta } = feed;
  const pick = (rule: string) => view.setFilters({ ...view.filters, rule: view.filters.rule === rule ? '' : rule });
  return (
    <Stack gap="md">
      <ProcessSummary procs={procs} meta={meta} now={now} />
      <RuleStrip groups={view.groups} rules={view.rules} procs={procs} rule={view.filters.rule} onPick={pick} kill={kill} />
      {procs.length === 0
        ? <EmptyState message={meta ? NONE_LISTED : NOT_SAMPLED} />
        : (
          <>
            <ProcessFilters filters={view.filters} onChange={view.setFilters} total={procs.length} look={view.look} rules={view.ruleNames} />
            <ProcessTable rows={view.rows} selectedId={view.selected?.id ?? null} onSelect={view.select} />
            {view.selected && <ProcessDetail proc={view.selected} kill={kill} />}
          </>
        )}
      <RulesTable rules={view.rules} />
    </Stack>
  );
};

export { ProcessesTab };
