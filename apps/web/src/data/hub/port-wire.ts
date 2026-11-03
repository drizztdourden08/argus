/* @layer renderer-app @kind logic */
import type { Wire } from './hub-transport.type';

const portWire = (port: MessagePort): Wire => ({
  post: (message) => port.postMessage(message),
  listen: (listener) => {
    port.addEventListener('message', (event) => listener(event.data));
    port.start();
  },
  close: () => port.close(),
});

export { portWire };
