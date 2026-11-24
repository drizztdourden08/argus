/* @layer renderer-app @kind component */
import { Card, Inline, ProgressBar, Stack, Status } from '@drizztdourden08/tessera';
import type { DragEvent } from 'react';
import { CURRENT_LABEL, CURRENT_TIP, METER_TONE } from '../accounts.constants';
import { accountKey, limitLabel, limitLine, mainLimit } from '../model';
import { DRAG_TYPE } from '../next.constants';
import type { AccountCardProps } from '../next.type';
import { AccountName } from './AccountName';

const toneOf = (severity: string | undefined) => METER_TONE[severity === 'critical' || severity === 'warning' ? severity : 'normal'];

const AccountCard = (props: AccountCardProps) => {
  const { account, onOpen } = props;
  const key = accountKey(account);
  const main = mainLimit(account);
  const line = limitLine(account);
  const active = account.active === true;
  const drag = (event: DragEvent) => event.dataTransfer.setData(DRAG_TYPE, key);
  return (
    <Card draggable onDragStart={drag} onActivate={() => onOpen(key)} edge={active ? 'left' : undefined} edgeTone={active ? 'info' : undefined}>
      <Stack gap="xs">
        <Inline gap="xs" align="center" wrap>
          <AccountName account={account} />
          {active && <Status tone="info" size="sm" dot title={CURRENT_TIP}>{CURRENT_LABEL}</Status>}
        </Inline>
        {main && <ProgressBar value={Math.round(main.percent ?? 0)} max={100} tone={toneOf(main.severity)} label={limitLabel(main)} formatValue={(value) => `${value}%`} showValue />}
        {line && <Status tone="danger" size="sm" dot>{line}</Status>}
      </Stack>
    </Card>
  );
};

export { AccountCard };
