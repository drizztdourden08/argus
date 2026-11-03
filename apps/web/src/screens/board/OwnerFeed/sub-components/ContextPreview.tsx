/* @layer renderer-app @kind component */
import { Button, Flex, Text } from '@drizztdourden08/tessera';
import { contextParts, hasContext } from '../../feed-model';
import { plainText } from '../../plain-text';
import { CONTEXT_LINES, CONTEXT_MORE_CHARS } from '../OwnerFeed.constants';
import type { ContextPreviewProps } from '../OwnerFeed.type';
import { ContextBlock } from './ContextBlock';

const ContextPreview = (props: ContextPreviewProps) => {
  const { entry, actions, onMore } = props;
  if (!hasContext(entry)) return <ContextBlock entry={entry} actions={actions} />;
  const context = entry.context ?? '';
  const text = plainText(context);
  const longer = contextParts(context).rest !== '' || text.length > CONTEXT_MORE_CHARS;
  return (
    <Flex direction="column" gap="2xs">
      <Text variant="caption" tone="dim" clamp={CONTEXT_LINES}>{text}</Text>
      {longer ? <Button size="xs" variant="ghost" onClick={onMore}>More</Button> : null}
    </Flex>
  );
};

export { ContextPreview };
