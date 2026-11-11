/* @layer renderer-app @kind component */
import { Inline, Tag, Text } from '@drizztdourden08/tessera';
import { handoverEstimate, handoverTitle } from '../model';
import { STATE_LABEL, STATE_TONE } from '../switch.constants';
import type { HandoverLineProps } from '../switch.type';

const HandoverLine = (props: HandoverLineProps) => {
  const { doc } = props;
  const state = doc.state ?? 'estimate_asked';
  return (
    <Inline gap="sm" align="center" wrap>
      <Text weight={600}>{handoverTitle(doc)}</Text>
      <Text variant="caption" tone="muted">{handoverEstimate(doc)}</Text>
      <Tag variant="urgency" color={STATE_TONE[state]}>{STATE_LABEL[state]}</Tag>
    </Inline>
  );
};

export { HandoverLine };
