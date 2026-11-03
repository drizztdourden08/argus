/* @layer renderer-app @kind types */
import type { HubMessage, TabMessage } from './hub.type';

type MessageListener = (message: HubMessage) => void;

interface TabHandle {
  tabId: string;
  send: (message: TabMessage) => void;
  onMessage: (listener: MessageListener) => () => void;
  close: () => void;
}

export type { MessageListener, TabHandle };
