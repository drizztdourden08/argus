/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { HeaderPopover } from '../../../../shell/HeaderPopover';
import { StatusNotice } from '../../../../shell';
import { useAlerts } from '../behavior/useAlerts';
import { ALERTS_HINT, NO_ALERTS } from '../BoardHeader.constants';
import type { BoardAlertsProps } from '../BoardHeader.type';
import { AlertSection } from './AlertSection';
import { AlertsLabel } from './AlertsLabel';
import { BoardFaults } from './BoardFaults';
import { BoardNotices } from './BoardNotices';
import { BoardStalls } from './BoardStalls';

const BoardAlerts = (props: BoardAlertsProps) => {
  const { boardKey, status } = props;
  const alerts = useAlerts(boardKey, status);
  return (
    <HeaderPopover trigger={<AlertsLabel count={alerts.count} tone={alerts.tone} />} label="Alerts" hint={ALERTS_HINT}>
      <StatusNotice status={status} />
      {alerts.count === 0 ? <Text variant="caption" tone="muted">{NO_ALERTS}</Text> : null}
      <AlertSection title="Stalled" count={alerts.stalls.length}>
        <BoardStalls stalls={alerts.stalls} boardKey={boardKey} />
      </AlertSection>
      <AlertSection title="Faults" count={alerts.faults.length}>
        <BoardFaults faults={alerts.faults} />
      </AlertSection>
      <AlertSection title="Notices" count={alerts.notices.length}>
        <BoardNotices notices={alerts.notices} />
      </AlertSection>
    </HeaderPopover>
  );
};

export { BoardAlerts };
