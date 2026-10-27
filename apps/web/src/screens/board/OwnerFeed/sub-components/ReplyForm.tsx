/* @layer renderer-app @kind component */
import { Button, Flex } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { OwnerBox } from '../../OwnerBox';
import { askCopy } from '../behavior/ask-copy';
import type { AskFormProps } from '../OwnerFeed.type';
import { DismissButton } from './DismissButton';
import { OptionButtons } from './OptionButtons';

const ReplyForm = (props: AskFormProps) => {
  const { entry, mode, actions } = props;
  const [text, setText] = useState('');
  const copy = askCopy(entry, mode);
  const busy = actions.isBusy(`answer:${entry.id}`);
  const send = () => void actions.reply(entry, copy.kind, text).then((taken) => taken && setText(''));
  return (
    <Flex direction="column" gap="xs">
      {mode === 'decide' ? <OptionButtons entry={entry} actions={actions} /> : null}
      <OwnerBox compact label={copy.label} value={text} onChange={setText} rows={2} onSubmit={send} disabled={busy} />
      <Flex gap="xs">
        <Button size="sm" variant={mode === 'answer' ? 'primary' : 'tertiary'} disabled={busy} onClick={send}>{copy.verb}</Button>
        {entry.kind === 'message' ? <DismissButton entry={entry} actions={actions} /> : null}
      </Flex>
    </Flex>
  );
};

export { ReplyForm };
