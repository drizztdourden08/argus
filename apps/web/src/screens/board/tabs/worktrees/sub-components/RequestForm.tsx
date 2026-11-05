/* @layer renderer-app @kind component */
import { Button, Callout, Flex, Text } from '@drizztdourden08/tessera';
import { BoardMarkdownEditor } from '../../../../../shell';
import { nameOf, threadOf } from '../model';
import { ASK_HINTS } from '../worktrees.constants';
import type { RequestFormProps } from '../worktrees-props.type';

const RequestForm = (props: RequestFormProps) => {
  const { row, panel, meta, controls, ask } = props;
  const thread = threadOf(row, meta) || 'the thread';
  const verb = panel.kind === 'delete' ? 'delete' : 'check on';
  return (
    <Flex direction="column" gap="sm" role="group" aria-label={`Ask ${thread} to ${verb} ${nameOf(row)}`}>
      <Text>Ask <Text.B>{thread}</Text.B> to {verb} <Text.Code>{row.path}</Text.Code>. The thread decides and answers here.</Text>
      <BoardMarkdownEditor
        aria-label="Message for the thread"
        value={panel.text}
        placeholder={panel.kind === 'delete' ? ASK_HINTS.delete : ASK_HINTS.check}
        rows={3}
        autoFocus
        layout="tabs"
        onChange={(text) => controls.patch({ text, error: null })}
        onSubmit={() => !ask.busy && void ask.send(row)}
        submitLabel={ask.busy ? 'Posting' : 'Send'}
        actions={<Button size="sm" variant="ghost" onClick={controls.close}>Cancel</Button>}
      />
      {panel.error && <Callout tone="danger">{panel.error}</Callout>}
    </Flex>
  );
};

export { RequestForm };
