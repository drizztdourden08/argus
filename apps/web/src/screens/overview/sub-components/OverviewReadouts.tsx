/* @layer renderer-app @kind component */
import { Icon, Inline, Tag, Text } from '@drizztdourden08/tessera';
import { PORTFOLIO_SCOPE, useDoc } from '../../../data/react';
import type { BoardMeta } from '../../../data/types';
import { Age, CALM_LOOK } from '../../../shell';
import { boardsTotals } from '../boards/behavior/boards-totals';
import { GOOD_LOOK, WARN_LOOK } from '../dashboards/dashboards.constants';
import type { OverviewAlertsProps } from '../overview.type';

const OverviewReadouts = (props: OverviewAlertsProps) => {
  const { entries, status } = props;
  const meta = useDoc<BoardMeta>(PORTFOLIO_SCOPE, 'meta', 'board');
  const totals = boardsTotals(entries);
  const updated = meta.doc ? <>updated <Age value={meta.doc.updatedAt} ago /></> : 'waiting for the boards';
  return (
    <Inline gap="sm" align="center">
      <Tag {...CALM_LOOK}>{`${totals.boards} boards`}</Tag>
      <Tag {...GOOD_LOOK} title="Leads running now">{`${totals.running} running`}</Tag>
      <Tag {...(totals.needs ? WARN_LOOK : CALM_LOOK)} title="Items waiting for you across all leads">{`${totals.needs} need you`}</Tag>
      <Inline gap="xs" align="center" title="When the boards last changed">
        <Icon name="clock" size={13} />
        <Text variant="caption">{status === 'unavailable' ? 'dashboard unreachable' : updated}</Text>
      </Inline>
    </Inline>
  );
};

export { OverviewReadouts };
