/* @layer renderer-app @kind test */
import { vi } from 'vitest';
import { Hub } from '../../src/data/hub/hub';
import type { HubMessage, MuxFrame, MuxHandlers, MuxOp, TabMessage } from '../../src/data/hub';
import type { StreamStatus } from '../../src/data/stream';

const createHubRig = (lingerMs?: number) => {
  const sent: MuxOp[][] = [];
  const posted = new Map<string, HubMessage[]>();
  const holder: { current?: MuxHandlers } = {};
  const handlers = (): MuxHandlers => {
    if (!holder.current) throw new Error('hub not connected');
    return holder.current;
  };
  const link = {
    open: vi.fn(),
    close: vi.fn(),
    send: (ops: readonly MuxOp[]) => sent.push([...ops]),
    getStatus: (): StreamStatus => 'idle',
  };
  const hub = new Hub({
    lingerMs,
    port: { post: (tabId, message) => posted.set(tabId, [...(posted.get(tabId) ?? []), message]) },
    connect: (given) => {
      holder.current = given;
      return link;
    },
  });
  return {
    hub,
    link,
    sent,
    to: (tabId: string): HubMessage[] => posted.get(tabId) ?? [],
    send: (tabId: string, message: TabMessage) => hub.receive(tabId, message),
    frame: (frame: MuxFrame) => handlers().onFrame(frame),
    status: (status: StreamStatus) => handlers().onStatus(status),
    resume: (): readonly MuxOp[] => handlers().resume(),
    opError: (op: MuxOp, error: Parameters<MuxHandlers['onOpError']>[1]) => handlers().onOpError(op, error),
    ops: (): MuxOp[] => sent.flat(),
  };
};

export { createHubRig };
