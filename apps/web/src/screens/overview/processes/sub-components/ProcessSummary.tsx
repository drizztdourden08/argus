/* @layer renderer-app @kind component */
import { Grid, StatTile } from '@drizztdourden08/tessera';
import { agoText, asNumber, formatMemory } from '../../format';
import { needsLook } from '../model';
import type { ProcessSummaryProps } from '../processes.type';
import { SampleNotices } from './SampleNotices';

const sumMemory = (procs: ProcessSummaryProps['procs']): number =>
  procs.reduce((sum, proc) => sum + (asNumber(proc.memMB) ?? 0), 0);

const ProcessSummary = (props: ProcessSummaryProps) => {
  const { procs, meta, now } = props;
  const look = procs.filter(needsLook).length;
  const count = asNumber(meta?.count) ?? procs.length;
  const total = asNumber(meta?.totalMemMB) ?? sumMemory(procs);
  return (
    <>
      <Grid minColWidth={160} gap="md">
        <StatTile size="sm" label={count === 1 ? 'Process' : 'Processes'} value={count} />
        <StatTile size="sm" label="Memory" value={formatMemory(total)} />
        <StatTile size="sm" label="Need a look" value={look} tone={look > 0 ? 'warning' : undefined} />
        <StatTile size="sm" label="Sampled" value={agoText(meta?.sampledAt, now) || 'not sampled yet'} />
      </Grid>
      <SampleNotices meta={meta} />
    </>
  );
};

export { ProcessSummary };
