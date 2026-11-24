/* @layer renderer-app @kind component */
import { Callout, EmptyState, Stack } from '@drizztdourden08/tessera';
import { useCacheReport } from './behavior/useCacheReport';
import { usePruneFlow } from './behavior/usePruneFlow';
import { NONE_CACHED, NO_FOLDER, READING } from './cache.constants';
import { CacheNotices } from './sub-components/CacheNotices';
import { CacheSummary } from './sub-components/CacheSummary';
import { CacheTable } from './sub-components/CacheTable';
import { PruneBar } from './sub-components/PruneBar';

const CacheTab = () => {
  const { report, error, reload } = useCacheReport();
  const flow = usePruneFlow(reload);
  if (!report) return error ? <Callout tone="danger">{error}</Callout> : <EmptyState message={READING} />;
  return (
    <Stack gap="md">
      <CacheSummary report={report} />
      <CacheNotices report={report} error={error} />
      <PruneBar report={report} flow={flow} />
      {report.rows.length === 0
        ? <EmptyState message={report.exists ? NONE_CACHED : NO_FOLDER} />
        : <CacheTable report={report} />}
    </Stack>
  );
};

export { CacheTab };
