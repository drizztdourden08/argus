/* @layer renderer-app @kind component */
import { Callout, Text } from '@drizztdourden08/tessera';
import { clipText } from '../../format';
import { ERROR_CLIP } from '../processes.constants';
import type { SampleNoticesProps } from '../processes.type';

const SampleNotices = (props: SampleNoticesProps) => {
  const { meta } = props;
  return (
    <>
      {meta?.platform ? <Text variant="caption" tone="muted">{meta.platform}</Text> : null}
      {meta?.error ? <Callout tone="danger">{`Sampling error: ${clipText(meta.error, ERROR_CLIP)}`}</Callout> : null}
      {meta?.rulesError ? <Callout tone="warning">{`The rules file has an error, the previous rules stay: ${clipText(meta.rulesError, ERROR_CLIP)}`}</Callout> : null}
    </>
  );
};

export { SampleNotices };
