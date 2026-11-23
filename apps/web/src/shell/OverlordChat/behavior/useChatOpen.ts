/* @layer renderer-app @kind logic */
import { useLocation, useNavigate, useSearchParams } from 'react-router';
import type { ChatOpen } from './chat-panels.type';
import { isChatOpen } from './chat-params';
import { withChat } from './with-chat';

const useChatOpen = (): ChatOpen => {
  const [params] = useSearchParams();
  const { hash } = useLocation();
  const navigate = useNavigate();
  const open = isChatOpen(params);
  const set = (next: boolean) => {
    void navigate({ search: `?${withChat(params, next).toString()}`, hash }, { replace: true });
  };
  return { open, toggle: () => set(!open), close: () => set(false) };
};

export { useChatOpen };
