/* @layer renderer-app @kind logic */
import { CHAT_PARAM } from '../OverlordChat.constants';

const withChat = (params: URLSearchParams, open: boolean): URLSearchParams => {
  const next = new URLSearchParams(params);
  if (open) next.set(CHAT_PARAM, '1');
  else next.delete(CHAT_PARAM);
  return next;
};

export { withChat };
