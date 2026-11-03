/* @layer renderer-app @kind test */
import { vi } from 'vitest';
import { FakeEventSource } from './fake-event-source';

const stubEventSource = (): FakeEventSource[] => {
  const streams: FakeEventSource[] = [];
  vi.stubGlobal('EventSource', class extends FakeEventSource {
    constructor(url: string) {
      super(url);
      streams.push(this);
    }
  });
  return streams;
};

export { stubEventSource };
