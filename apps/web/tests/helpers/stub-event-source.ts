/* @layer renderer-app @kind test */
import { vi } from 'vitest';
import { FakeSource } from './fake-source';

const stubEventSource = (): FakeSource[] => {
  const streams: FakeSource[] = [];
  vi.stubGlobal('EventSource', class extends FakeSource {
    constructor(url: string) {
      super(url);
      streams.push(this);
    }
  });
  return streams;
};

export { stubEventSource };
