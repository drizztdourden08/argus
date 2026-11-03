/* @layer renderer-app @kind types */
interface BuildRow {
  id: string;
  name: string;
  version: string;
  commit: string;
  checkout: string;
  packedAt: string;
  size: number;
}

export type { BuildRow };
