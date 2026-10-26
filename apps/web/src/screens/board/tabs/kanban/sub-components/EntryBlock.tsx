/* @layer renderer-app @kind component */
import { Flex, InlineEdit } from '@drizztdourden08/tessera';
import { BoardMarkdown } from '../../../../../shell';
import { askMode, hasContext } from '../../../feed-model';
import { AskForm, ContextBlock } from '../../../OwnerFeed';
import type { EntryBlockProps } from '../drawer-props.type';
import { validateDecision } from '../model';
import { EntryHead } from './EntryHead';
import { EntryReply } from './EntryReply';

const EntryBlock = (props: EntryBlockProps) => {
  const { row, feed, ops, now } = props;
  const { entry } = row;
  const editable = entry.kind === 'decision' && entry.author === 'human' && entry.status === 'pending';
  const showContext = entry.author !== 'human' && (row.open || hasContext(entry));
  return (
    <Flex direction="column" gap="xs">
      <EntryHead entry={entry} now={now} />
      {showContext && <ContextBlock entry={entry} actions={feed} />}
      {editable ? (
        <InlineEdit
          multiline
          label={`decision ${entry.id}`}
          placeholder="decision text"
          value={entry.text}
          validate={validateDecision}
          onCommit={(value) => void ops.editRecord({ c: 'entries', id: entry.id }, { field: 'text', prev: entry.text, value: value.trim() }, `Saved ${entry.id}.`)}
        />
      ) : (
        <BoardMarkdown size="sm" source={entry.text} />
      )}
      {row.replies.map((reply) => (
        <EntryReply key={reply.id} reply={reply} now={now} />
      ))}
      {row.open && <AskForm entry={entry} mode={askMode(entry)} actions={feed} />}
    </Flex>
  );
};

export { EntryBlock };
