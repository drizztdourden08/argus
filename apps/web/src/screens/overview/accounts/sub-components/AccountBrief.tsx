/* @layer renderer-app @kind component */
import { Inline, ProgressBar, Stack, Status } from '@drizztdourden08/tessera';
import { CURRENT_LABEL, CURRENT_TIP, METER_TONE } from '../accounts.constants';
import type { AccountNameProps } from '../cards.type';
import { limitLabel, limitLine, mainLimit } from '../model';
import { AccountName } from './AccountName';

const toneOf = (severity: string | undefined) => METER_TONE[severity === 'critical' || severity === 'warning' ? severity : 'normal'];

const AccountBrief = (props: AccountNameProps) => {
  const { account } = props;
  const main = mainLimit(account);
  const line = limitLine(account);
  const active = account.active === true;
  return (
    <Stack gap="xs">
      <AccountName account={account} />
      {main && <ProgressBar value={Math.round(main.percent ?? 0)} max={100} tone={toneOf(main.severity)} label={limitLabel(main)} formatValue={(value) => `${value}%`} showValue />}
      {(active || line) && (
        <Inline gap="xs" align="center" wrap>
          {active && <Status tone="info" size="sm" dot title={CURRENT_TIP}>{CURRENT_LABEL}</Status>}
          {line && <Status tone="danger" size="sm" dot>{line}</Status>}
        </Inline>
      )}
    </Stack>
  );
};

export { AccountBrief };
