/* @layer renderer-app @kind logic */
import type { TabLink } from '../hub/tab-link';
import { ScopeStream } from './scope-stream';
import type { ScopeStreams } from './stream.type';

const createScopeStreams = (link: TabLink): ScopeStreams => {
  const streams = new Map<string, ScopeStream>();
  return {
    get(scope) {
      const known = streams.get(scope);
      if (known) return known;
      const created = new ScopeStream(scope, link);
      streams.set(scope, created);
      return created;
    },
  };
};

export { createScopeStreams };
