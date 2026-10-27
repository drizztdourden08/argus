/* @layer renderer-app @kind component */
import { Inline } from '@drizztdourden08/tessera';
import { asNumber, formatMemory } from '../../../../format';
import { needsLook } from '../model';
import type { ProcessSummaryProps } from '../processes.type';
import { ProcessFigure } from './ProcessFigure';
import { SampleFigures } from './SampleFigures';
import { SampleNotices } from './SampleNotices';

const sumMemory = (procs: ProcessSummaryProps['procs']): number =>
  procs.reduce((sum, proc) => sum + (asNumber(proc.memMB) ?? 0), 0);

const ProcessSummary = (props: ProcessSummaryProps) => {
  const { procs, meta } = props;
  if (!meta && procs.length === 0) return null;
  const look = procs.filter(needsLook).length;
  const count = asNumber(meta?.count) ?? procs.length;
  const total = asNumber(meta?.totalMemMB) ?? sumMemory(procs);
  return (
    <>
      <Inline gap="md" align="baseline" wrap role="status">
        <ProcessFigure value={count} after={count === 1 ? 'process' : 'processes'} />
        <ProcessFigure before="memory" value={formatMemory(total)} />
        {look > 0 && <ProcessFigure value={look} after="need a look" />}
        <SampleFigures meta={meta} />
      </Inline>
      <SampleNotices meta={meta} />
    </>
  );
};

export { ProcessSummary };
