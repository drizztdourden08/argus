/* @layer renderer-app @kind component */
import { Flex, SegmentedControl, Text } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { OWNER_TEXT_MAX } from '../../../../../data/posts';
import { BoardMarkdownEditor } from '../../../../../shell';
import { textProblem } from '../../../owner-posts';
import type { NoteKind } from '../../../owner-posts';
import { COMPOSE_KINDS } from '../kanban.constants';
import type { ComposerProps } from '../drawer-props.type';

const Composer = (props: ComposerProps) => {
  const { item, acts } = props;
  const [kind, setKind] = useState<NoteKind>('decision');
  const [text, setText] = useState('');
  const word = kind === 'decision' ? 'decision' : 'note';
  const send = () => void acts.compose(item, kind, text).then((taken) => taken && setText(''));
  return (
    <Flex direction="column" gap="xs">
      <Flex gap="sm" align="center" wrap>
        <SegmentedControl<NoteKind> selectedTone="neutral" label="Post as" size="sm" value={kind} options={COMPOSE_KINDS} onChange={setKind} />
        <Text variant="caption" tone="muted">
          {kind === 'decision' ? `queued for ${item.thread} to act on` : `more context for ${item.thread}`}
        </Text>
      </Flex>
      <BoardMarkdownEditor
        label={`The ${word} for ${item.thread}`}
        error={textProblem(text)}
        value={text}
        onChange={setText}
        onSubmit={send}
        submitLabel={`Post ${word}`}
        maxLength={OWNER_TEXT_MAX}
        rows={2}
        layout="tabs"
        disabled={acts.isBusy(`compose:${item.id}`)}
      />
    </Flex>
  );
};

export { Composer };
