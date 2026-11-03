/* @layer renderer-app @kind logic */
import type { HubEnvelope } from './hub-transport.type';
import type { HubMessage, TabMessage } from './hub.type';

const envelope = <M extends TabMessage | HubMessage>(tab: string, epoch: number, msg: M): HubEnvelope<M> => ({ tab, epoch, msg });

export { envelope };
