/* @layer renderer-app @kind component */
import { Button, Stack, Text } from '@drizztdourden08/tessera';
import { movesText, SWITCH_ACTION } from '../accounts.constants';
import type { SwitchActionProps } from '../cards.type';
import { blockReason } from '../model';

const SwitchAction = (props: SwitchActionProps) => {
  const { account, current, state, switching, confirming, onSwitch } = props;
  const reason = blockReason(state, switching);
  const from = current ? (current.label ?? current.key ?? current.id) : null;
  return (
    <Stack gap="2xs">
      <Button variant="primary" disabled={reason !== '' || confirming} onClick={() => onSwitch(account.key ?? account.id)}>{SWITCH_ACTION}</Button>
      {reason && <Text variant="caption" tone="warning">{reason}</Text>}
      <Text variant="caption" tone="muted">{movesText(from, account.label ?? account.key ?? account.id)}</Text>
    </Stack>
  );
};

export { SwitchAction };
