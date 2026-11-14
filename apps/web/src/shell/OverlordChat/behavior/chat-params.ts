/* @layer renderer-app @kind logic */
import { CHAT_PARAM } from '../OverlordChat.constants';

const isChatOpen = (params: URLSearchParams): boolean => params.get(CHAT_PARAM) === '1';

export { isChatOpen };
