/* @layer renderer-app @kind component */
import { Button, Callout, Inline, Text } from '@drizztdourden08/tessera';
import { PORTFOLIO_SCOPE, useCollection } from '../../../../../data/react';
import type { AccountDoc } from '../../../../../data/types';
import { ACCOUNTS_COLL } from '../../accounts.constants';
import { useCountdown } from '../behavior/useCountdown';
import { useSwitchActions } from '../behavior/useSwitchActions';
import { useSwitchFeed } from '../behavior/useSwitchFeed';
import { countdownText, toMs } from '../model';
import { CANCEL_LABEL, NOW_LABEL, REASON_OWNER, REASON_RULE } from '../switch.constants';

const SwitchCountdown = () => {
  const { doc } = useSwitchFeed();
  const accounts = useCollection<AccountDoc>(PORTFOLIO_SCOPE, ACCOUNTS_COLL).docs;
  const actions = useSwitchActions();
  const left = useCountdown(toMs(doc?.countdownEndsAt));
  if (doc?.phase !== 'countdown') return null;
  const target = accounts.find((account) => account.id === doc.to);
  const buttons = (
    <Inline gap="sm">
      <Button size="sm" disabled={actions.busy} onClick={() => actions.start(doc.to ?? '', false)}>{NOW_LABEL}</Button>
      <Button size="sm" variant="secondary" disabled={actions.busy} onClick={actions.cancel}>{CANCEL_LABEL}</Button>
    </Inline>
  );
  return (
    <Callout tone="warning" action={buttons}>
      <Inline gap="sm" align="center" wrap>
        <Text weight={600}>{`Switching Desktop to ${target?.label ?? doc.to} in ${countdownText(left)}`}</Text>
        <Text variant="caption" tone="muted">{doc.startedBy === 'rule' ? REASON_RULE : REASON_OWNER}</Text>
      </Inline>
    </Callout>
  );
};

export { SwitchCountdown };
