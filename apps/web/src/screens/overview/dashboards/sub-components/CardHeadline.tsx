/* @layer renderer-app @kind component */
import { DateText, Inline, Tag, Text } from '@drizztdourden08/tessera';
import { formatAgo, orText } from '../../format';
import { STATE_LOOK, WARN_LOOK } from '../dashboards.constants';
import type { CardHeadlineProps } from '../dashboards.type';
import { attentionOf, leadState } from '../model';

const seenTip = (card: CardHeadlineProps['card'], now: number): string => {
  const coordinator = card.coordinator;
  if (!coordinator) return `No live lead session; board updated ${formatAgo(card.updatedAt, now)} ago`;
  const activity = coordinator.activity ? ` (${coordinator.activity})` : '';
  return `Lead ${coordinator.session ?? ''} last seen ${formatAgo(coordinator.lastSeen, now)} ago${activity}`;
};

const CardHeadline = (props: CardHeadlineProps) => {
  const { card, now } = props;
  const state = leadState(card, now);
  const needs = attentionOf(card).needs;
  const seen = card.coordinator?.lastSeen ?? card.updatedAt;
  const needsTip = orText(card.attention?.slice(0, 4).join('\n'), `${needs} item(s) waiting for you`);
  return (
    <Inline gap="sm" align="center" wrap>
      <Tag {...STATE_LOOK[state]}>{state}</Tag>
      {state !== 'not started' && (
        <Text variant="caption" tone="muted" title={seenTip(card, now)}>{seen ? <DateText value={seen} format="relative" /> : 'never'}</Text>
      )}
      {needs > 0 && <Tag {...WARN_LOOK} title={needsTip}>{`${needs} need you`}</Tag>}
    </Inline>
  );
};

export { CardHeadline };
