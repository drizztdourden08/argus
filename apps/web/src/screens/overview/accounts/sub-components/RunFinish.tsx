/* @layer renderer-app @kind component */
import { Button, Callout, Inline, Stack } from '@drizztdourden08/tessera';
import { FAILED_TEXT, MATCH_TEXT } from '../compare.constants';
import type { RunFinishProps } from '../compare.type';

const RunFinish = (props: RunFinishProps) => {
  const { phase, result, busy, onUndo, onBack } = props;
  if (phase !== 'done' && phase !== 'failed') return null;
  return (
    <Stack gap="sm">
      {phase === 'done'
        ? <Callout tone="success" title={MATCH_TEXT}>{result ?? MATCH_TEXT}</Callout>
        : <Callout tone="danger" title="Sync failed">{result ?? FAILED_TEXT}</Callout>}
      <Inline gap="sm" align="center">
        <Button size="sm" variant="ghost" onClick={onBack}>Back</Button>
        <Button size="sm" disabled={busy} onClick={onUndo}>Undo</Button>
      </Inline>
    </Stack>
  );
};

export { RunFinish };
