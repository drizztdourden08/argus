/* @layer renderer-app @kind test */
import type { Wire } from '../../src/data/hub/hub-transport.type';

interface Delivery {
  to: (data: unknown) => void;
  data: unknown;
}

interface Peer {
  listener: ((data: unknown) => void) | null;
}

class FakeNetwork {
  private readonly wires = new Map<string, Set<Peer>>();
  private pending: Delivery[] = [];

  open = (name: string): Wire => {
    const self: Peer = { listener: null };
    const group: Set<Peer> = this.wires.get(name) ?? new Set<Peer>();
    group.add(self);
    this.wires.set(name, group);
    return {
      post: (message) => {
        for (const peer of group) {
          if (peer !== self) this.pending.push({ to: (data) => peer.listener?.(data), data: structuredClone(message) });
        }
      },
      listen: (listener) => {
        self.listener = listener;
      },
      close: () => {
        group.delete(self);
      },
    };
  };

  flush(): void {
    while (this.pending.length > 0) {
      const batch = this.pending;
      this.pending = [];
      for (const { to, data } of batch) to(data);
    }
  }

  size(name: string): number {
    return this.wires.get(name)?.size ?? 0;
  }
}

export { FakeNetwork };
