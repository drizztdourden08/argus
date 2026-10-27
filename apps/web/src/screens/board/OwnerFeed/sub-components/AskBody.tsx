/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { askMode } from '../../feed-model';
import { plainText } from '../../plain-text';
import type { EntryBodyProps } from '../OwnerFeed.type';
import { AskForm } from './AskForm';
import { ContextBlock } from './ContextBlock';

const AskBody = (props: EntryBodyProps) => {
  const { entry, actions } = props;
  return (
    <Flex direction="column" gap="sm">
      <ContextBlock entry={entry} actions={actions} />
      <Text weight={600} title={entry.text}>{plainText(entry.text)}</Text>
      <AskForm entry={entry} mode={askMode(entry)} actions={actions} />
    </Flex>
  );
};

export { AskBody };
