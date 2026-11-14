/* @layer renderer-app @kind logic */
import { useSearchParams } from 'react-router';
import { CHAT_PARAM } from '../overlord.constants';

const useChatOpen = (): { open: boolean; toggle: () => void; close: () => void } => {
  const [params, setParams] = useSearchParams();
  const open = params.get(CHAT_PARAM) === '1';
  const set = (next: boolean) => {
    const rest = new URLSearchParams(params);
    if (next) rest.set(CHAT_PARAM, '1');
    else rest.delete(CHAT_PARAM);
    setParams(rest, { replace: true });
  };
  return { open, toggle: () => set(!open), close: () => set(false) };
};

export { useChatOpen };
