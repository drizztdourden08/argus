/* @layer renderer-app @kind types */
type DocData = Readonly<Record<string, unknown>>;

interface DocRow {
  id: string;
  data: DocData;
}

interface ListResult {
  seq: number;
  docs: DocRow[];
}

interface DocResult {
  seq: number;
  id: string;
  exists: boolean;
  data: DocData | null;
}

type QueryOp = '==' | '!=' | '<' | '<=' | '>' | '>=' | 'in' | 'contains';

interface DbQuery {
  where?: [string, QueryOp, unknown][];
  orderBy?: [string, 'asc' | 'desc'];
  limit?: number;
}

interface BatchOp {
  op: 'set' | 'update' | 'delete';
  coll: string;
  id: string;
  data?: DocData;
}

interface ApiErrorBody {
  code: string;
  message: string;
  details?: unknown;
}

interface PostResult {
  id: string;
  status: string;
  outcome: string | null;
  entry: DocData | null;
  seq: number;
}

interface BatchResult {
  results: (PostResult | { error: ApiErrorBody })[];
}

interface ProjectSummary {
  key: string;
  name: string;
  root: string | null;
  url: string;
  hasBoard: boolean;
  running: number;
}

interface RequestOptions {
  method?: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';
  json?: unknown;
  raw?: Blob;
  rawType?: string;
}

export type {
  ApiErrorBody,
  BatchOp,
  BatchResult,
  DbQuery,
  DocData,
  DocResult,
  ListResult,
  PostResult,
  ProjectSummary,
  RequestOptions,
};
