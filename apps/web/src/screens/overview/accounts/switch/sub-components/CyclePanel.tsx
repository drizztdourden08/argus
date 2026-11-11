/* @layer renderer-app @kind component */
import { Card, Stack, Text } from '@drizztdourden08/tessera';
import { useSwitchFeed } from '../behavior/useSwitchFeed';
import { notAsked } from '../model';
import { CYCLE_TITLE, NOT_ASKED_TEXT } from '../switch.constants';
import type { CyclePanelProps } from '../switch.type';
import { HandoverLine } from './HandoverLine';

const CyclePanel = (props: CyclePanelProps) => {
  const { cycle, handovers } = useSwitchFeed();
  if (!cycle || !props.account.active) return null;
  const idle = notAsked(cycle.leads, handovers);
  const facts = [cycle.askAt === undefined ? '' : `asked at ${cycle.askAt}%`, cycle.k === undefined ? '' : `k ${cycle.k}`].filter(Boolean);
  return (
    <Card>
      <Stack gap="xs">
        <Text weight={600}>{`${CYCLE_TITLE} ${cycle.id ?? ''}`}</Text>
        {facts.length > 0 && <Text variant="caption" tone="muted">{facts.join(' · ')}</Text>}
        {handovers.map((doc) => <HandoverLine key={doc.id} doc={doc} />)}
        {idle > 0 && <Text variant="caption" tone="muted">{`${idle} idle, ${NOT_ASKED_TEXT}`}</Text>}
      </Stack>
    </Card>
  );
};

export { CyclePanel };
