/* @layer renderer-app @kind component */
import { Box, Stack } from '@drizztdourden08/tessera';
import { detectDone } from '../model';
import type { ProcessDocProps } from '../switch.type';
import { StageTimeline } from './StageTimeline';
import { SyncBlock } from './SyncBlock';
import { ThreadList } from './ThreadList';

const ProcessDetail = (props: ProcessDocProps) => {
  const { doc } = props;
  return (
    <Box className="process-panel__body">
      <StageTimeline doc={doc} />
      <Stack gap="sm">
        <ThreadList threads={doc.threads ?? []} checked={detectDone(doc)} />
        <SyncBlock sync={doc.sync} />
      </Stack>
    </Box>
  );
};

export { ProcessDetail };
