/* @layer renderer-app @kind component */
import { Badge, Inline, Spacer, Text } from '@drizztdourden08/tessera';
import { resolveAvatar } from '../../../../avatar';
import { orText } from '../../../../format';
import { LeadMark } from '../../../../shell/records';
import { DOT_COLOR } from '../dashboards.constants';
import type { CardHeadlineProps } from '../dashboards.type';
import { attentionOf } from '../model';

const CardHeadline = (props: CardHeadlineProps) => {
  const { card, state } = props;
  const needs = attentionOf(card).needs;
  const needsTip = orText(card.attention?.slice(0, 4).join('\n'), `${needs} item(s) waiting for you`);
  const mark = resolveAvatar(card.key);
  return (
    <Inline gap="sm" align="center">
      {mark && <LeadMark source={mark} label={`${card.name} lead`} />}
      <Badge variant="dot" color={DOT_COLOR[state]} />
      <Text variant="title" size="lead" title={card.name}>{card.name}</Text>
      <Spacer />
      {needs > 0 && <Badge variant="number" color="warning" icon="triangle-alert" value={needs} label={needsTip} />}
    </Inline>
  );
};

export { CardHeadline };
