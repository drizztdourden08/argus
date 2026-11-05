/* @layer renderer-app @kind types */
type Raw = Record<string, unknown>;

interface RunSource {
  id: string;
  type?: string | null;
  stage?: string | null;
  fields?: Raw;
  createdAt?: string;
  createdBy?: string | null;
  startedAt?: string | null;
  endedAt?: string | null;
  updatedAt?: string;
  deletedAt?: string | null;
}

export type { Raw, RunSource };
