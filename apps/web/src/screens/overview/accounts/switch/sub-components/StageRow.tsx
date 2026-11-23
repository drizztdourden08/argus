/* @layer renderer-app @kind component */
import { Box, Stack, Status, Text } from '@drizztdourden08/tessera';
import { STEP_STATUSES } from '../switch.constants';
import type { StageRowProps } from '../switch.type';
import { CountdownLine } from './CountdownLine';

const StageRow = (props: StageRowProps) => {
  const { view, doc } = props;
  const counting = view.stage === 'countdown' && doc.phase === 'countdown';
  return (
    <Box as="li" className="stage-row" data-state={view.state} aria-current={view.state === 'running' ? 'step' : undefined}>
      <Status map={STEP_STATUSES} value={view.state} size="sm" dot />
      <Text weight={600}>{view.label}</Text>
      <Stack gap="2xs" className="stage-row__text">
        {counting ? <CountdownLine doc={doc} /> : <Text tone={view.state === 'waiting' ? 'muted' : undefined}>{view.text}</Text>}
        {view.detail && <Text variant="caption" tone="muted">{view.detail}</Text>}
      </Stack>
      <Text variant="caption" tone="muted" numeric smartTimes={false}>{view.at}</Text>
    </Box>
  );
};

export { StageRow };
