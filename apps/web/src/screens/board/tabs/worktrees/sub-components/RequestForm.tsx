/* @layer renderer-app @kind component */
import { Button, Callout, Flex, Text } from '@drizztdourden08/tessera';
import { BoardMarkdownEditor } from '../../../../../shell';
import type { KeyboardEvent } from 'react';
import { nameOf, threadOf } from '../model';
import { ASK_HINTS } from '../worktrees.constants';
import type { RequestFormProps } from '../worktrees-props.type';

const RequestForm = (props: RequestFormProps) => {
  const { row, panel, meta, controls, ask } = props;
  const thread = threadOf(row, meta) || 'the thread';
  const verb = panel.kind === 'delete' ? 'delete' : 'check on';
  const onKeyDown = (event: KeyboardEvent<HTMLElement>) => {
    if (event.key !== 'Enter' || !(event.ctrlKey || event.metaKey)) return;
    event.preventDefault();
    if (!ask.busy) void ask.send(row);
  };
  return (
    <Flex direction="column" gap="sm" role="group" aria-label={`Ask ${thread} to ${verb} ${nameOf(row)}`} onKeyDown={onKeyDown}>
      <Text>Ask <Text.B>{thread}</Text.B> to {verb} <Text.Code>{row.path}</Text.Code>. The thread decides and answers here.</Text>
      <BoardMarkdownEditor
        aria-label="Message for the thread"
        value={panel.text}
        placeholder={panel.kind === 'delete' ? ASK_HINTS.delete : ASK_HINTS.check}
        rows={3}
        layout="tabs"
        onChange={(text) => controls.patch({ text, error: null })}
      />
      {panel.error && <Callout tone="danger">{panel.error}</Callout>}
      <Flex gap="sm" align="center" justify="between" wrap>
        <Text variant="caption" tone="faint">Ctrl+Enter sends. Enter adds a line.</Text>
        <Flex gap="xs">
          <Button size="sm" variant="ghost" onClick={controls.close}>Cancel</Button>
          <Button size="sm" variant="primary" disabled={ask.busy} onClick={() => void ask.send(row)}>{ask.busy ? 'Posting' : 'Send'}</Button>
        </Flex>
      </Flex>
    </Flex>
  );
};

export { RequestForm };
