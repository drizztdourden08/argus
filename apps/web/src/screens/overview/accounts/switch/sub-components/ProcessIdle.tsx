/* @layer renderer-app @kind component */
import { Disclosure, Inline, Stack, Status, Text } from '@drizztdourden08/tessera';
import { resultLine } from '../model';
import { LAST_RUN, NO_SWITCH } from '../switch.constants';
import type { ProcessPanelProps } from '../switch.type';
import { ProcessDetail } from './ProcessDetail';

const ProcessIdle = (props: ProcessPanelProps) => {
  const { doc, accounts } = props;
  return (
    <Stack gap="xs">
      <Inline gap="sm" align="center" wrap>
        <Status tone="neutral" size="sm" dot>{NO_SWITCH}</Status>
        <Text variant="caption" tone={doc?.phase === 'failed' ? 'danger' : 'muted'}>{resultLine(doc, accounts)}</Text>
      </Inline>
      {doc && (doc.steps ?? []).length > 0 && <Disclosure size="sm" summary={LAST_RUN}><ProcessDetail doc={doc} /></Disclosure>}
    </Stack>
  );
};

export { ProcessIdle };
