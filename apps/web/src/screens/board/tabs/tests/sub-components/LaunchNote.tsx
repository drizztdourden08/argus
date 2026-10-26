/* @layer renderer-app @kind component */
import { Disclosure, Markdown, Text } from '@drizztdourden08/tessera';
import { useTestsShared } from '../behavior/useTestsShared';
import { plainText } from '../../../plain-text';
import { NOTE_FOLD_CHARS } from '../tests.constants';
import type { LaunchNoteProps } from '../tests-props.type';

const foldedLine = (note: string): string => {
  const line = plainText(note);
  return line.length > NOTE_FOLD_CHARS ? `${line.slice(0, NOTE_FOLD_CHARS)}...` : line;
};

const LaunchNote = (props: LaunchNoteProps) => {
  const { tag, note } = props;
  const { folds } = useTestsShared();
  const key = `${tag}#note`;
  const open = folds.isOpen(key, false);
  return (
    <Disclosure
      size="sm"
      open={open}
      onOpenChange={(next) => folds.set(key, next)}
      summary={open ? 'Note' : <Text variant="caption" tone="muted">{foldedLine(note)}</Text>}
    >
      {open && <Markdown size="sm">{note}</Markdown>}
    </Disclosure>
  );
};

export { LaunchNote };
