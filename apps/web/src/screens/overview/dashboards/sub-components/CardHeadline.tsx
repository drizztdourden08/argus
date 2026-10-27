/* @layer renderer-app @kind component */
import { Badge, Inline, Spacer, Text } from '@drizztdourden08/tessera';
import { orText } from '../../../../format';
import { DOT_COLOR } from '../dashboards.constants';
import type { CardHeadlineProps } from '../dashboards.type';
import { attentionOf } from '../model';

const CardHeadline = (props: CardHeadlineProps) => {
  const { card, state } = props;
  const needs = attentionOf(card).needs;
  const needsTip = orText(card.attention?.slice(0, 4).join('\n'), `${needs} item(s) waiting for you`);
  return (
    <Inline gap="sm" align="center">
      <Badge variant="dot" color={DOT_COLOR[state]} />
      <Text variant="title" title={card.name}>{card.name}</Text>
      <Spacer />
      {needs > 0 && <Badge variant="number" color="warning" icon="triangle-alert" value={needs} label={needsTip} />}
    </Inline>
  );
};

export { CardHeadline };
