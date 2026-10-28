/* @layer renderer-app @kind component */
import { Button, Icon, Inline, Tag, Text } from '@drizztdourden08/tessera';
import { useRef } from 'react';
import { PORTFOLIO_SCOPE, useDoc } from '../../../../data/react';
import type { BoardMeta } from '../../../../data/types';
import { useNow } from '../../clock';
import { Age } from '../../../../shell';
import { CALM_LOOK, GOOD_LOOK, WARN_LOOK } from '../dashboards.constants';
import type { DashboardTotalsProps } from '../dashboards.type';
import { totalsOf } from '../model';
import { SendDownPopover } from './SendDownPopover';

const DashboardTotals = (props: DashboardTotalsProps) => {
  const { cards, status, send } = props;
  const anchor = useRef<HTMLButtonElement>(null);
  const now = useNow();
  const meta = useDoc<BoardMeta>(PORTFOLIO_SCOPE, 'meta', 'board');
  const totals = totalsOf(cards, now);
  const updated = meta.doc ? <>updated <Age value={meta.doc.updatedAt} ago /></> : 'waiting for the cards';
  return (
    <Inline gap="md" align="center" justify="between" wrap>
      <Inline gap="sm" align="center" wrap>
        <Tag {...GOOD_LOOK} title="Leads running now">{`${totals.running} running`}</Tag>
        <Tag {...(totals.needs ? WARN_LOOK : CALM_LOOK)} title="Items waiting for you across all leads">
          <Inline gap="xs" align="center">
            <Icon name="triangle-alert" size={12} />
            {`${totals.needs} need you`}
          </Inline>
        </Tag>
        <Inline gap="xs" align="center" title="When the cards last changed">
          <Icon name="clock" size={13} />
          <Text variant="caption">{status === 'unavailable' ? 'dashboard unreachable' : updated}</Text>
        </Inline>
      </Inline>
      <Button ref={anchor} size="sm" icon={<Icon name="send" />} onClick={() => send.open('all')} title="Send a decision or a note to every lead">Send to all</Button>
      <SendDownPopover send={send} owner="all" anchorRef={anchor} />
    </Inline>
  );
};

export { DashboardTotals };
