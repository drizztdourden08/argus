/* @layer renderer-app @kind logic */
import { ScopeStream } from './scope-stream';
import type { OpenSource, ScopeStreams } from './stream.type';

const browserSource: OpenSource = (url) => new EventSource(url);

const createScopeStreams = (openSource: OpenSource = browserSource): ScopeStreams => {
  const streams = new Map<string, ScopeStream>();
  return {
    get(scope) {
      const known = streams.get(scope);
      if (known) return known;
      const created = new ScopeStream(scope, openSource);
      streams.set(scope, created);
      return created;
    },
  };
};

export { createScopeStreams };
