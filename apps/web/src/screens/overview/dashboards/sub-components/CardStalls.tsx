/* @layer renderer-app @kind component */
import { Inline, Tag } from '@drizztdourden08/tessera';
import { stallsOf, stallText, useStalls } from '../../../../data/stalls';
import { formatAgo } from '../../../../format';
import { BAD_LOOK } from '../../../../shell';
import type { CardStallsProps } from '../dashboards.type';

const CardStalls = (props: CardStallsProps) => {
  const { card, now } = props;
  const stalls = stallsOf(useStalls().docs, card.key);
  if (stalls.length === 0) return null;
  return (
    <Inline gap="xs" align="center" wrap>
      {stalls.map((stall) => (
        <Tag key={stall.id} pill {...BAD_LOOK} title={`open ${formatAgo(stall.since, now)}`}>{`stalled: ${stallText(stall.text)}`}</Tag>
      ))}
    </Inline>
  );
};

export { CardStalls };
