/* @layer renderer-app @kind component */
import { Icon, Inline, Spacer, Status, Text } from '@drizztdourden08/tessera';
import { formatAgo } from '../../../../format';
import { ShortAge } from '../../sub-components/ShortAge';
import { STATE_TONE } from '../dashboards.constants';
import type { CardStateProps } from '../dashboards.type';

const seenTip = (card: CardStateProps['card'], now: number): string => {
  const coordinator = card.coordinator;
  if (!coordinator) return `No live lead session; board updated ${formatAgo(card.updatedAt, now)} ago`;
  const activity = coordinator.activity ? ` (${coordinator.activity})` : '';
  return `Lead ${coordinator.session ?? ''} last seen ${formatAgo(coordinator.lastSeen, now)} ago${activity}`;
};

const CardState = (props: CardStateProps) => {
  const { card, state, now } = props;
  const seen = card.coordinator?.lastSeen ?? card.updatedAt;
  return (
    <Inline gap="sm" align="center">
      <Status variant="pill" size="sm" tone={STATE_TONE[state]}>{state}</Status>
      <Spacer />
      {state !== 'not started' && (
        <Inline gap="2xs" align="center" title={seenTip(card, now)}>
          <Icon name="clock" size={12} />
          <Text variant="caption">{seen ? <ShortAge value={seen} /> : 'never'}</Text>
        </Inline>
      )}
    </Inline>
  );
};

export { CardState };
