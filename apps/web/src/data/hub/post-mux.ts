/* @layer renderer-app @kind logic */
import { MUX_URL } from './hub.constants';
import type { MuxOp, MuxOpResult, MuxPostResult } from './hub.type';

const resultsOf = async (response: Response): Promise<MuxOpResult[]> => {
  if (response.status !== 200) return [];
  const body = (await response.json().catch(() => null)) as { results?: MuxOpResult[] } | null;
  return body?.results ?? [];
};

const postMux = async (conn: string, ops: MuxOp[]): Promise<MuxPostResult> => {
  const response = await fetch(`${MUX_URL}/${encodeURIComponent(conn)}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ ops }),
  });
  return { status: response.status, results: await resultsOf(response) };
};

export { postMux };
