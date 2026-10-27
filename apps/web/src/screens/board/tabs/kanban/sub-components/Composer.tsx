/* @layer renderer-app @kind component */
import { Button, Flex, SegmentedControl, Text } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { OwnerBox } from '../../../OwnerBox';
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
        <SegmentedControl<NoteKind> label="Post as" size="sm" value={kind} options={COMPOSE_KINDS} onChange={setKind} />
        <Text variant="caption" tone="muted">
          {kind === 'decision' ? `queued for ${item.thread} to act on` : `more context for ${item.thread}`}
        </Text>
      </Flex>
      <OwnerBox label={`The ${word} for ${item.thread}`} value={text} onChange={setText} rows={2} onSubmit={send} disabled={acts.isBusy(`compose:${item.id}`)} />
      <Flex>
        <Button size="sm" variant="primary" disabled={acts.isBusy(`compose:${item.id}`)} onClick={send}>{`Post ${word}`}</Button>
      </Flex>
    </Flex>
  );
};

export { Composer };
