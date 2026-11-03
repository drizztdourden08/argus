/* @layer renderer-app @kind types */
import type { Wire } from './hub-transport.type';

interface Member {
  wire: Wire;
  unwatch: () => void;
  seen: number;
}

export type { Member };
