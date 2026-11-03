/* @layer renderer-app @kind entry */
import { Hub } from './hub';
import { portWire } from './port-wire';
import { WorkerHost } from './worker-host';

const host = new WorkerHost({
  locks: 'locks' in navigator ? navigator.locks : null,
  createHub: (port) => new Hub({ port }),
  now: Date.now,
});

const onConnect = (event: MessageEvent): void => {
  const [port] = event.ports;
  if (port) host.connect(portWire(port));
};

Object.assign(self, { onconnect: onConnect });
