/* @layer renderer-app @kind component */
import { Button, Callout, DialogShell, DropZone, Flex, Text, Textarea, toast } from '@drizztdourden08/tessera';
import type { ClipboardEvent, KeyboardEvent } from 'react';
import { IMAGE_TYPES } from '../../../../../data/api';
import { useNoteDraft } from '../behavior/useNoteDraft';
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
  const onKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if (event.key !== 'Enter' || !(event.ctrlKey || event.metaKey)) return;
    event.preventDefault();
    if (highlighted) draft.post(highlighted);
    else toast({ variant: 'warning', message: 'Choose pass, fail, partial or blocked with its button.' });
  };
  const onPaste = (event: ClipboardEvent<HTMLTextAreaElement>) => {
    const file = Array.from(event.clipboardData.files)[0];
    if (!file) return;
    event.preventDefault();
    draft.setShot(file);
  };
  const actions = (
    <>
      <Button variant="ghost" onClick={onClose}>Cancel</Button>
      {OUTCOMES.map((outcome) => (
        <Button
          key={outcome.id}
          variant={outcome.id === highlighted ? 'primary' : 'secondary'}
          disabled={draft.busy}
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
        <Textarea
          rows={3}
          value={draft.text}
          placeholder={COMPOSE_HINTS.fail}
          aria-label={`Result note for ${test.id}`}
          onChange={(event) => draft.setText(event.target.value)}
          onKeyDown={onKeyDown}
          onPaste={onPaste}
          autoFocus
        />
        {test.proofs.length > 0 && <ScreenshotGallery shots={test.proofs} />}
        {draft.shot && <ShotPreview file={draft.shot} testId={test.id} onRemove={draft.removeShot} />}
        <DropZone variant="inline" label="Attach screenshot" accept={IMAGE_TYPES} onDrop={(files) => files[0] && draft.setShot(files[0])} />
        {draft.error && <Callout tone="danger">{draft.error}</Callout>}
      </Flex>
    </DialogShell>
  );
};

export { NoteDialog };
