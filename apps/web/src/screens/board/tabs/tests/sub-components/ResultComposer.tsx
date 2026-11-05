/* @layer renderer-app @kind component */
import { Box, Button, Callout, DropZone, Flex } from '@drizztdourden08/tessera';
import { IMAGE_TYPES } from '../../../../../data/api';
import { OWNER_TEXT_MAX } from '../../../../../data/posts';
import { BoardMarkdownEditor } from '../../../../../shell';
import { overLimitKeys } from '../behavior/over-limit-keys';
import { pasteShot } from '../behavior/paste-shot';
import { noteProblem } from '../model';
import { COMPOSE_HINTS } from '../tests.constants';
import type { ResultComposerProps } from '../tests-props.type';
import { ShotPreview } from './ShotPreview';

const ResultComposer = (props: ResultComposerProps) => {
  const { testId, outcome, controls } = props;
  const problem = noteProblem(outcome, controls.text);
  const onPaste = pasteShot(controls.setShot);
  return (
    <Flex direction="column" gap="xs" role="group" aria-label={`${outcome} note for ${testId}`}>
      <Box onPaste={onPaste} onKeyDown={overLimitKeys(controls.text)}>
        <BoardMarkdownEditor
          aria-label={`Note for ${testId}`}
          placeholder={COMPOSE_HINTS[outcome]}
          value={controls.text}
          onChange={controls.setText}
          onSubmit={controls.send}
          submitLabel={`Send ${outcome}`}
          submitDisabled={Boolean(problem) || controls.busy}
          onEscape={controls.cancel}
          actions={<Button size="sm" variant="ghost" onClick={controls.cancel}>Cancel</Button>}
          error={controls.text.length > 0 ? problem : null}
          maxLength={OWNER_TEXT_MAX}
          rows={3}
          layout="tabs"
          autoFocus
        />
      </Box>
      {controls.shot && <ShotPreview file={controls.shot} testId={testId} onRemove={controls.removeShot} />}
      <DropZone variant="inline" label="Attach screenshot" accept={IMAGE_TYPES} onDrop={(files) => files[0] && controls.setShot(files[0])} />
      {controls.error && <Callout tone="danger">{controls.error}</Callout>}
    </Flex>
  );
};

export { ResultComposer };
