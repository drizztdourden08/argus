/* @layer renderer-app @kind component */
import { Button, Callout, Disclosure, Flex, Text } from '@drizztdourden08/tessera';
import { BoardMarkdown } from '../../../../shell';
import { contextParts, hasContext } from '../../feed-model';
import type { EntryBodyProps } from '../OwnerFeed.type';

const ContextBlock = (props: EntryBodyProps) => {
  const { entry, actions } = props;
  if (!hasContext(entry)) {
    if (entry.kind !== 'question') return null;
    return (
      <Callout
        tone="warning"
        variant="footnote"
        action={
          <Button size="xs" variant="ghost" disabled={actions.isBusy(`askctx:${entry.id}`)} onClick={() => void actions.askContext(entry)}>
            Ask for context
          </Button>
        }
      >
        No context was given: ask the thread what this is about
      </Callout>
    );
  }
  const parts = contextParts(entry.context ?? '');
  return (
    <Flex direction="column" gap="xs">
      <Text variant="overline" tone="muted">Context</Text>
      <Text as="div" tone="dim"><BoardMarkdown size="sm" source={parts.shown} /></Text>
      {parts.rest ? (
        <Disclosure summary="Show the rest" size="sm">
          <BoardMarkdown size="sm" source={parts.rest} />
        </Disclosure>
      ) : null}
    </Flex>
  );
};

export { ContextBlock };
