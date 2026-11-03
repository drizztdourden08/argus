/* @layer renderer-app @kind component */
import { Disclosure, Flex } from '@drizztdourden08/tessera';
import { askCopy } from '../behavior/ask-copy';
import type { AskFormProps } from '../OwnerFeed.type';
import { DismissButton } from './DismissButton';
import { OptionButtons } from './OptionButtons';
import { ReplyForm } from './ReplyForm';

const AnswerBlock = (props: AskFormProps) => {
  const { entry, mode, actions } = props;
  if (mode === 'info' && entry.kind !== 'message') return <DismissButton entry={entry} actions={actions} />;
  const form = <ReplyForm entry={entry} mode={mode} actions={actions} quick={false} />;
  return (
    <Flex direction="column" gap="xs">
      {mode === 'decide' ? <OptionButtons entry={entry} actions={actions} /> : null}
      {entry.kind === 'message' ? <Flex gap="xs"><DismissButton entry={entry} actions={actions} /></Flex> : null}
      {mode === 'answer' ? form : <Disclosure summary={askCopy(entry, mode).fold} size="sm">{form}</Disclosure>}
    </Flex>
  );
};

export { AnswerBlock };
