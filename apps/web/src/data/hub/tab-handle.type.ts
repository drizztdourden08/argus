/* @layer renderer-app @kind types */
import type { HubMessage, TabMessage } from './hub.type';

interface TabHandle {
  tabId: string;
  send(message: TabMessage): void;
  onMessage(listener: (message: HubMessage) => void): () => void;
  close(): void;
}

export type { TabHandle };
