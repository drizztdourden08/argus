/* @layer renderer-app @kind component */
import type { AskFormProps } from '../behavior/ask.type';
import { DismissButton } from './DismissButton';
import { ReplyForm } from './ReplyForm';

const AskForm = (props: AskFormProps) => {
  const { entry, mode, actions } = props;
  if (mode === 'info' && entry.kind !== 'message') return <DismissButton entry={entry} actions={actions} />;
  return <ReplyForm entry={entry} mode={mode} actions={actions} />;
};

export { AskForm };
