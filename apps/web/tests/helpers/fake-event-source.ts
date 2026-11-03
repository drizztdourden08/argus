/* @layer renderer-app @kind test */
import type { SourceLike } from '../../src/data/stream';

type Handler = (event: MessageEvent<string>) => void;

type OpenSource = (url: string) => SourceLike;

class FakeEventSource implements SourceLike {
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

interface FakeEventSources {
  open: OpenSource;
  sources: FakeEventSource[];
}

const createFakeEventSources = (): FakeEventSources => {
  const sources: FakeEventSource[] = [];
  const open: OpenSource = (url) => {
    const source = new FakeEventSource(url);
    sources.push(source);
    return source;
  };
  return { open, sources };
};

export { createFakeEventSources, FakeEventSource };
