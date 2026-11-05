/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { OwnerBox } from '../../OwnerBox';
import { askCopy } from '../behavior/ask-copy';
import type { AskFormProps } from '../OwnerFeed.type';
import { DismissButton } from './DismissButton';
import { OptionButtons } from './OptionButtons';

const ReplyForm = (props: AskFormProps) => {
  const { entry, mode, actions, quick = true } = props;
  const [text, setText] = useState('');
  const copy = askCopy(entry, mode);
  const busy = actions.isBusy(`answer:${entry.id}`);
  const send = () => void actions.reply(entry, copy.kind, text).then((taken) => taken && setText(''));
  return (
    <Flex direction="column" gap="xs">
      {quick && mode === 'decide' ? <OptionButtons entry={entry} actions={actions} /> : null}
      <OwnerBox compact ariaLabel={copy.label} placeholder={copy.label} value={text} onChange={setText} rows={3} onSubmit={send} disabled={busy} submitLabel={copy.verb} />
      {quick && entry.kind === 'message' ? <Flex gap="xs"><DismissButton entry={entry} actions={actions} /></Flex> : null}
    </Flex>
  );
};

export { ReplyForm };
