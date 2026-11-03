/* @layer renderer-app @kind logic */
import { fetchHealth } from './fetch-health';
import type { MuxHandlers } from './hub.type';
import { MuxConnection } from './mux-connection';
import { postMux } from './post-mux';

const createMuxConnection = (handlers: MuxHandlers): MuxConnection =>
  new MuxConnection({ ...handlers, openSource: (url) => new EventSource(url), post: postMux, health: fetchHealth });

export { createMuxConnection };
