/* @layer renderer-app @kind test */
import type { OpenSource, SourceLike } from '../../src/data/stream';

type Handler = (event: MessageEvent<string>) => void;

class FakeSource implements SourceLike {
  readonly url: string;
  closed = false;
  private readonly handlers = new Map<string, Handler[]>();

  constructor(url: string) {
    this.url = url;
  }

  addEventListener(type: string, listener: Handler): void {
    this.handlers.set(type, [...(this.handlers.get(type) ?? []), listener]);
  }

  close(): void {
    this.closed = true;
  }

  emit(type: string, body?: unknown): void {
    const event = { data: JSON.stringify(body ?? {}) } as MessageEvent<string>;
    for (const handler of this.handlers.get(type) ?? []) handler(event);
  }
}

interface FakeSources {
  open: OpenSource;
  sources: FakeSource[];
}

const createFakeSources = (): FakeSources => {
  const sources: FakeSource[] = [];
  const open: OpenSource = (url) => {
    const source = new FakeSource(url);
    sources.push(source);
    return source;
  };
  return { open, sources };
};

export { createFakeSources };
export type { FakeSource };
