/* @layer renderer-app @kind component */
import { Stack, Text } from '@drizztdourden08/tessera';
import { detectDone, isActive, stageViews } from '../model';
import type { StepBodyProps } from '../switch.type';
import { CountdownLine } from './CountdownLine';
import { SyncBlock } from './SyncBlock';
import { ThreadList } from './ThreadList';

const StepBody = (props: StepBodyProps) => {
  const { doc, stage } = props;
  if (stage === 'countdown' && doc.phase === 'countdown') return <CountdownLine doc={doc} />;
  if (stage === 'handover') return <ThreadList threads={doc.threads ?? []} checked={detectDone(doc)} skippable={isActive(doc)} />;
  if (stage === 'sync') return <SyncBlock sync={doc.sync} />;
  const view = stageViews(doc).find((candidate) => candidate.stage === stage);
  return (
    <Stack gap="2xs">
      <Text tone={view?.state === 'waiting' ? 'muted' : undefined}>{view?.text ?? ''}</Text>
      {view?.detail && <Text variant="caption" tone="muted">{view.detail}</Text>}
    </Stack>
  );
};

export { StepBody };
