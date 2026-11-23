/* @layer renderer-app @kind component */
import { Inline, Stack, Status, Text } from '@drizztdourden08/tessera';
import { processTitle } from '../model';
import { BY_OWNER, BY_RULE, PHASE_STATUSES, STUCK_PREFIX } from '../switch.constants';
import type { ProcessSummaryProps } from '../switch.type';

const ProcessSummary = (props: ProcessSummaryProps) => {
  const { doc, accounts } = props;
  return (
    <Stack gap="2xs">
      <Inline gap="sm" align="center" wrap>
        <Status map={PHASE_STATUSES} value={doc.phase} fallback="idle" size="sm" dot role="status" />
        <Text variant="caption" tone="muted">{doc.startedBy === 'rule' ? BY_RULE : BY_OWNER}</Text>
      </Inline>
      <Text weight={600}>{processTitle(doc, accounts)}</Text>
      {doc.blocker && doc.phase !== 'countdown' && <Text variant="caption" tone="warning" role="status">{`${STUCK_PREFIX} ${doc.blocker}`}</Text>}
      {doc.error && <Text variant="caption" tone="danger">{doc.error}</Text>}
    </Stack>
  );
};

export { ProcessSummary };
