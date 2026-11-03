/* @layer renderer-app @kind test */
import { vi } from 'vitest';
import { MuxConnection } from '../../src/data/hub';
import type { MuxFrame, MuxOp, MuxOpResult } from '../../src/data/hub';
import type { StreamStatus } from '../../src/data/stream';
import { createFakeSources } from './fake-source';

interface SentPost {
  conn: string;
  ops: MuxOp[];
}

interface Reply {
  status: number;
  results?: MuxOpResult[];
}

const createMuxRig = (resume: MuxOp[] = []) => {
  const fake = createFakeSources();
  const posts: SentPost[] = [];
  const reply: Reply = { status: 200 };
  const frames: MuxFrame[] = [];
  const statuses: StreamStatus[] = [];
  const health = vi.fn().mockResolvedValue(200);
  const onOpError = vi.fn();
  const post = async (conn: string, ops: MuxOp[]) => {
    posts.push({ conn, ops });
    return { status: reply.status, results: reply.results ?? ops.map((): MuxOpResult => ({ ok: true })) };
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

const settle = (): Promise<void> => vi.advanceTimersByTimeAsync(0);

export { createMuxRig, settle };
