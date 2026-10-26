/* @layer renderer-app @kind component */
import { Box, Button, Callout, DialogShell, DropZone, Flex, Text, toast } from '@drizztdourden08/tessera';
import { IMAGE_TYPES } from '../../../../../data/api';
import { OwnerBox } from '../../../OwnerBox';
import { textProblem } from '../../../owner-posts';
import { useNoteDraft } from '../behavior/useNoteDraft';
import { pasteShot } from '../behavior/paste-shot';
import { useTestsShared } from '../behavior/useTestsShared';
import { ScreenshotGallery } from '../../../screenshots';
import { noteOutcome } from '../model';
import { COMPOSE_HINTS, NOTE_DIALOG_TEXT, OUTCOMES } from '../tests.constants';
import type { NoteDialogProps } from '../tests-drawer-props.type';
import { ShotPreview } from './ShotPreview';

const NoteDialog = (props: NoteDialogProps) => {
  const { test, onClose } = props;
  const { boardKey } = useTestsShared();
  const draft = useNoteDraft(boardKey, test.id, onClose);
  const highlighted = noteOutcome(test);
  const over = textProblem(draft.text);
  const onPaste = pasteShot(draft.setShot);
  const submit = () => {
    if (over) toast({ variant: 'warning', message: over });
    else if (highlighted) draft.post(highlighted);
    else toast({ variant: 'warning', message: 'Choose pass, fail, partial or blocked with its button.' });
  };
  const actions = (
    <>
      <Button variant="ghost" onClick={onClose}>Cancel</Button>
      {OUTCOMES.map((outcome) => (
        <Button
          key={outcome.id}
          variant={outcome.id === highlighted ? 'primary' : 'secondary'}
          disabled={draft.busy || over !== null}
          aria-label={`Post ${test.id} as ${outcome.id}`}
          onClick={() => draft.post(outcome.id)}
        >
          {outcome.id}
        </Button>
      ))}
    </>
  );
  return (
    <DialogShell open onClose={onClose} title={`Add a note to ${test.id}`} size="md" actions={actions}>
      <Flex direction="column" gap="sm">
        <Text tone="muted">{NOTE_DIALOG_TEXT}</Text>
        <Box onPaste={onPaste}>
          <OwnerBox
            ariaLabel={`Result note for ${test.id}`}
            placeholder={COMPOSE_HINTS.fail}
            value={draft.text}
            onChange={draft.setText}
            onSubmit={submit}
            rows={3}
            autoFocus
          />
        </Box>
        {test.proofs.length > 0 && <ScreenshotGallery shots={test.proofs} />}
        {draft.shot && <ShotPreview file={draft.shot} testId={test.id} onRemove={draft.removeShot} />}
        <DropZone variant="inline" label="Attach screenshot" accept={IMAGE_TYPES} onDrop={(files) => files[0] && draft.setShot(files[0])} />
        {draft.error && <Callout tone="danger">{draft.error}</Callout>}
      </Flex>
    </DialogShell>
  );
};

export { NoteDialog };
