/* @layer renderer-app @kind test */
import { vi } from 'vitest';
import { MuxConnection } from '../../src/data/hub/mux-connection';
import type { MuxFrame, MuxOp, MuxOpResult, MuxPostResult } from '../../src/data/hub';
import type { StreamStatus } from '../../src/data/stream';
import { createFakeEventSources } from './fake-event-source';
import type { FakeEventSource } from './fake-event-source';

interface SentPost {
  conn: string;
  ops: MuxOp[];
}

interface Reply {
  status: number;
  results?: MuxOpResult[];
}

const createMuxRig = (resume: MuxOp[] = []) => {
  const fake = createFakeEventSources();
  const posts: SentPost[] = [];
  const reply: Reply = { status: 200 };
  const frames: MuxFrame[] = [];
  const statuses: StreamStatus[] = [];
  const health = vi.fn().mockResolvedValue(200);
  const onOpError = vi.fn();
  const post = (conn: string, ops: MuxOp[]): Promise<MuxPostResult> => {
    posts.push({ conn, ops });
    return Promise.resolve({ status: reply.status, results: reply.results ?? ops.map((): MuxOpResult => ({ ok: true })) });
  };
  const mux = new MuxConnection({
    openSource: fake.open,
    post,
    health,
    resume: () => resume,
    onFrame: (frame) => frames.push(frame),
    onStatus: (status) => statuses.push(status),
    onOpError,
  });
  return { mux, sources: fake.sources, posts, reply, frames, statuses, health, onOpError };
};

const sub = (scope: string, ...colls: string[]): MuxOp => ({ op: 'sub', scope, colls });

const hello = (source: FakeEventSource | undefined, conn: string): void => source?.emit('hello', { conn, v: 1 });

const settle = async (): Promise<void> => {
  await vi.advanceTimersByTimeAsync(0);
};

export { createMuxRig, hello, settle, sub };
