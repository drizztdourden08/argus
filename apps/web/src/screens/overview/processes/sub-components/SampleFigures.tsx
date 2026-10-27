/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { asNumber } from '../../../../format';
import { ShortAge } from '../../sub-components/ShortAge';
import type { SampleNoticesProps } from '../processes.type';
import { ProcessFigure } from './ProcessFigure';

const sampleTitle = (meta: NonNullable<SampleNoticesProps['meta']>): string => {
  const took = asNumber(meta.sampleMs);
  return `${meta.sampledAt ?? ''}${took === null ? '' : ` (the sample took ${Math.round(took)} ms)`}`;
};

const SampleFigures = (props: SampleNoticesProps) => {
  const { meta } = props;
  return (
    <>
      {meta?.sampledAt
        ? <ProcessFigure before="sampled" value={<ShortAge value={meta.sampledAt} />} after="ago" title={sampleTitle(meta)} />
        : <Text variant="subtitle">not sampled yet</Text>}
      {meta?.platform ? <Text variant="subtitle">{meta.platform}</Text> : null}
    </>
  );
};

export { SampleFigures };
