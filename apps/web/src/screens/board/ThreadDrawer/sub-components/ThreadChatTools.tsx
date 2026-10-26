/* @layer renderer-app @kind component */
import { CopyButton, Link } from '@drizztdourden08/tessera';
import type { ThreadChatToolsProps } from '../ThreadDrawer.type';

const ThreadChatTools = (props: ThreadChatToolsProps) => {
  const { thread } = props;
  return (
    <>
      {thread.link && <Link href={thread.link} variant="subtle" external>Open the chat</Link>}
      {thread.link && <CopyButton text={thread.link} label="Copy the session link" variant="ghost" size="xs" />}
      <CopyButton text={thread.id} label={`Copy id ${thread.id}`} variant="ghost" size="xs" />
    </>
  );
};

export { ThreadChatTools };
