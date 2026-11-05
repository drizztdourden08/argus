/* @layer renderer-app @kind component */
import { Icon, Inline, Text } from '@drizztdourden08/tessera';
import { HeaderPopover } from '../../../shell/HeaderPopover';
import { StatusNotice } from '../../../shell';
import { ProjectLink } from '../projects';
import type { OverviewAlertsProps } from '../overview.type';

const OverviewAlerts = (props: OverviewAlertsProps) => {
  const { entries, status } = props;
  const live = entries.filter((entry) => !entry.record.archived);
  const faults = live.filter((entry) => entry.live.fault);
  const stalled = live.filter((entry) => entry.live.stalls > 0);
  const count = faults.length + stalled.length;
  return (
    <HeaderPopover
      trigger={<Inline gap="xs" align="center"><Icon name="triangle-alert" size={13} /><Text variant="caption">{`Alerts ${count}`}</Text></Inline>}
      label="Alerts"
      hint="Wake faults, closed boards and stalls across all boards"
    >
      <StatusNotice status={status} />
      {count === 0 ? <Text variant="caption" tone="muted">Nothing needs a look.</Text> : null}
      {faults.map(({ record, live: state }) => (
        <Inline key={`fault-${record.id}`} gap="xs" align="center">
          <ProjectLink project={record.key} />
          <Text variant="caption">{state.fault?.reason === 'board-closed' ? 'board closed' : 'wake failed'}</Text>
        </Inline>
      ))}
      {stalled.map(({ record, live: state }) => (
        <Inline key={`stall-${record.id}`} gap="xs" align="center">
          <ProjectLink project={record.key} hash="#activity" />
          <Text variant="caption">{`${state.stalls} stalled`}</Text>
        </Inline>
      ))}
    </HeaderPopover>
  );
};

export { OverviewAlerts };
