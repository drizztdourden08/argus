/* @layer renderer-app @kind component */
import { Button, Flex } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { askCopy } from '../behavior/ask-copy';
import type { AskFormProps } from '../OwnerFeedAsk.type';
import { DismissButton } from './DismissButton';
import { OptionButtons } from './OptionButtons';
import { ReplyForm } from './ReplyForm';

const AnswerBlock = (props: AskFormProps) => {
  const { entry, mode, actions } = props;
  const [custom, setCustom] = useState(false);
  if (mode === 'info' && entry.kind !== 'message') return <Flex><DismissButton entry={entry} actions={actions} /></Flex>;
  const form = <ReplyForm entry={entry} mode={mode} actions={actions} quick={false} />;
  if (mode === 'answer') return form;
  return (
    <Flex direction="column" gap="xs">
      <Flex gap="xs" wrap align="center">
        {mode === 'decide' ? <OptionButtons entry={entry} actions={actions} /> : null}
        {entry.kind === 'message' ? <DismissButton entry={entry} actions={actions} /> : null}
        <Button size="sm" variant="ghost" aria-expanded={custom} onClick={() => setCustom(!custom)}>
          {mode === 'decide' ? 'Custom answer' : askCopy(entry, mode).fold}
        </Button>
      </Flex>
      {custom ? form : null}
    </Flex>
  );
};

export { AnswerBlock };
