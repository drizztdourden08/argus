/* @layer renderer-app @kind component */
import { Stack, Text } from '@drizztdourden08/tessera';
import { stageViews } from '../model';
import type { StepBodyProps } from '../switch.type';

const TextBody = (props: StepBodyProps) => {
  const { doc, stage } = props;
  const view = stageViews(doc).find((candidate) => candidate.stage === stage);
  return (
    <Stack gap="2xs">
      <Text tone={view?.state === 'waiting' ? 'muted' : undefined}>{view?.text ?? ''}</Text>
      {view?.detail && <Text variant="caption" tone="muted">{view.detail}</Text>}
    </Stack>
  );
};

export { TextBody };
