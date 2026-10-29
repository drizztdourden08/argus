/* @layer renderer-app @kind component */
import { Callout, Stack } from '@drizztdourden08/tessera';
import { SHADOW_TEXT } from '../cache.constants';
import type { CacheNoticesProps } from '../cache.type';

const CacheNotices = (props: CacheNoticesProps) => {
  const { report, error } = props;
  const problems = report.problems.length > 0;
  if (!report.shadow && !problems && !error) return null;
  return (
    <Stack gap="xs">
      {report.shadow && <Callout tone="info">{SHADOW_TEXT}</Callout>}
      {problems && <Callout tone="warning">{`A lockfile could not be read, so nothing is deleted until it can: ${report.problems.join('; ')}`}</Callout>}
      {error && <Callout tone="danger">{error}</Callout>}
    </Stack>
  );
};

export { CacheNotices };
