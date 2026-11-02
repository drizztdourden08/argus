/* @layer renderer-app @kind types */
import type { RequestedBy, WaitsOn } from '../deps';

interface Criterion {
  n: number;
  text: string;
  done: boolean;
}

interface WorkItem {
  id: string;
  title: string;
  status: string;
  thread: string;
  subject?: string;
  description?: string;
  ac?: Criterion[];
  rank?: number;
  updatedAt?: string;
  archivedAt?: string;
  deletedAt?: string;
  waitsOn?: WaitsOn | WaitsOn[];
  requestedBy?: RequestedBy | RequestedBy[];
}

export type { WorkItem };
