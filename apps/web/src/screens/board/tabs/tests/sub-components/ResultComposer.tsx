/* @layer renderer-app @kind component */
import { Button, Callout, DropZone, Flex, Text, Textarea } from '@drizztdourden08/tessera';
import type { ClipboardEvent, KeyboardEvent } from 'react';
import { IMAGE_TYPES, noteProblem } from '../model';
import { COMPOSE_HINTS } from '../tests.constants';
import type { ResultComposerProps } from '../tests-props.type';
import { ShotPreview } from './ShotPreview';

const ResultComposer = (props: ResultComposerProps) => {
  const { testId, outcome, controls } = props;
  const problem = noteProblem(outcome, controls.text);
  const onKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if (event.key === 'Enter' && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      if (!problem && !controls.busy) controls.send();
    } else if (event.key === 'Escape') controls.cancel();
  };
  const onPaste = (event: ClipboardEvent<HTMLTextAreaElement>) => {
    const file = Array.from(event.clipboardData.files)[0];
    if (!file) return;
    event.preventDefault();
    controls.setShot(file);
  };
  return (
    <Flex direction="column" gap="xs" role="group" aria-label={`${outcome} note for ${testId}`}>
      <Textarea
        rows={3}
        value={controls.text}
        placeholder={COMPOSE_HINTS[outcome]}
        aria-label={`Note for ${testId}`}
        onChange={(event) => controls.setText(event.target.value)}
        onKeyDown={onKeyDown}
        onPaste={onPaste}
      />
      {controls.shot && <ShotPreview file={controls.shot} testId={testId} onRemove={controls.removeShot} />}
      <DropZone variant="inline" label="Attach screenshot" accept={IMAGE_TYPES} onDrop={(files) => files[0] && controls.setShot(files[0])} />
      {controls.error && <Callout tone="danger">{controls.error}</Callout>}
      <Flex gap="sm" align="center" justify="between" wrap>
        {problem && controls.text.length > 0
          ? <Text variant="caption" tone="warning">{problem}</Text>
          : <Text variant="caption" tone="faint">Ctrl+Enter sends. Enter adds a line.</Text>}
        <Flex gap="xs">
          <Button size="sm" variant="ghost" onClick={controls.cancel}>Cancel</Button>
          <Button size="sm" variant="primary" disabled={Boolean(problem) || controls.busy} onClick={controls.send}>{`Send ${outcome}`}</Button>
        </Flex>
      </Flex>
    </Flex>
  );
};

export { ResultComposer };
