/* @layer renderer-app @kind component */
import { Button, DateText, Inline, Tag, Text } from '@drizztdourden08/tessera';
import { PORTFOLIO_SCOPE, useDoc } from '../../../../data/react';
import type { BoardMeta } from '../../../../data/types';
import { useNow } from '../../clock';
import { CALM_LOOK, GOOD_LOOK, WARN_LOOK } from '../dashboards.constants';
import type { DashboardTotalsProps } from '../dashboards.type';
import { totalsOf } from '../model';

const DashboardTotals = (props: DashboardTotalsProps) => {
  const { cards, status, onSend } = props;
  const now = useNow();
  const meta = useDoc<BoardMeta>(PORTFOLIO_SCOPE, 'meta', 'board');
  const totals = totalsOf(cards, now);
  const updated = meta.doc ? <>updated <DateText value={meta.doc.updatedAt} format="relative" /></> : 'waiting for the cards';
  return (
    <Inline gap="sm" align="center" wrap>
      <Tag {...GOOD_LOOK} title="Leads running now">{`${totals.running} running`}</Tag>
      <Tag {...(totals.needs ? WARN_LOOK : CALM_LOOK)} title="Items waiting for you across all leads">{`${totals.needs} need you`}</Tag>
      <Text variant="caption" tone="muted" title="When the cards last changed">{status === 'unavailable' ? 'dashboard unreachable' : updated}</Text>
      <Button size="sm" onClick={() => onSend('all')} title="Send a decision or a note to every lead">Send to all</Button>
    </Inline>
  );
};

export { DashboardTotals };
