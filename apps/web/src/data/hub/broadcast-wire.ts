/* @layer renderer-app @kind logic */
import type { Wire } from './hub-transport.type';

const openBroadcastWire = (name: string): Wire => {
  const channel = new BroadcastChannel(name);
  return {
    post: (message) => channel.postMessage(message),
    listen: (listener) => channel.addEventListener('message', (event) => listener(event.data)),
    close: () => channel.close(),
  };
};

export { openBroadcastWire };
