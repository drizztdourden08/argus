/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { useRecordPane } from '../../../../shell';
import { askMode } from '../../feed-model';
import type { EntryBodyProps } from '../OwnerFeed.type';
import { AnswerBlock } from './AnswerBlock';
import { ContextPreview } from './ContextPreview';
import { EntryImages } from './EntryImages';

const AskBody = (props: EntryBodyProps) => {
  const { entry, actions } = props;
  const pane = useRecordPane();
  return (
    <Flex direction="column" gap="sm">
      <ContextPreview entry={entry} actions={actions} onMore={() => pane.open(entry.id)} />
      <EntryImages entry={entry} />
      <AnswerBlock entry={entry} mode={askMode(entry)} actions={actions} />
    </Flex>
  );
};

export { AskBody };
