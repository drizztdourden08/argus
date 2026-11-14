/* @layer renderer-app @kind logic */
import { useSearchParams } from 'react-router';
import type { ChatOpen } from '../OverlordChat.type';
import { isChatOpen } from './chat-params';
import { withChat } from './with-chat';

const useChatOpen = (): ChatOpen => {
  const [params, setParams] = useSearchParams();
  const open = isChatOpen(params);
  const set = (next: boolean) => setParams(withChat(params, next), { replace: true });
  return { open, toggle: () => set(!open), close: () => set(false) };
};

export { useChatOpen };
