/* @layer renderer-app @kind component */
import { Icon, Inline, Stack, Status, Text } from '@drizztdourden08/tessera';
import { accountName } from '../../model';
import { BY_OWNER, BY_RULE, PHASE_STATUSES, STUCK_PREFIX } from '../switch.constants';
import type { ProcessSummaryProps } from '../switch.type';

const ProcessSummary = (props: ProcessSummaryProps) => {
  const { doc, accounts } = props;
  return (
    <Stack gap="2xs">
      <Inline gap="sm" align="center" wrap>
        <Status map={PHASE_STATUSES} value={doc.phase} fallback="idle" size="sm" dot role="status" />
        <Inline gap="xs" align="center">
          <Text weight={600}>{accountName(accounts, doc.from)}</Text>
          <Icon name="arrow-right" size={14} aria-hidden />
          <Text weight={600}>{accountName(accounts, doc.to)}</Text>
        </Inline>
        <Text variant="caption" tone="muted">{doc.startedBy === 'rule' ? BY_RULE : BY_OWNER}</Text>
      </Inline>
      {doc.blocker && doc.phase !== 'countdown' && <Text variant="caption" tone="warning" role="status">{`${STUCK_PREFIX} ${doc.blocker}`}</Text>}
      {doc.error && <Text variant="caption" tone="danger">{doc.error}</Text>}
    </Stack>
  );
};

export { ProcessSummary };
