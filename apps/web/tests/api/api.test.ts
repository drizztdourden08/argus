/* @layer renderer-app @kind test */
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ApiError, batchDocs, getDoc, listDocs, listProjects, queryDocs, setDoc } from '../../src/data/api';
import { attachProof, postEntry, recordResult, sendDown } from '../../src/data/posts';
import { listRuns, sendStdin, startRun, stopRun } from '../../src/data/runs';

const fetchMock = vi.fn();

const reply = (body: unknown, status = 200) => fetchMock.mockResolvedValueOnce(new Response(JSON.stringify(body), { status }));

const lastCall = () => {
  const [path, init] = fetchMock.mock.calls.at(-1) as [string, RequestInit | undefined];
  return { path, method: init?.method, body: init?.body, headers: init?.headers as Record<string, string> | undefined };
};

vi.stubGlobal('fetch', fetchMock);

afterEach(() => fetchMock.mockReset());

describe('K2 db routes', { tags: ['critical'] }, () => {
  it('lists, gets, queries, sets and batches', async () => {
    reply({ seq: 1, docs: [] });
    await listDocs('rotp', 'items');
    expect(lastCall()).toMatchObject({ path: '/api/p/rotp/db/items', method: 'GET' });

    reply({ seq: 1, id: 'I-1', exists: true, data: { id: 'I-1' } });
    expect((await getDoc('rotp', 'items', 'I-1')).exists).toBe(true);
    expect(lastCall().path).toBe('/api/p/rotp/db/items/I-1');

    reply({ seq: 1, docs: [] });
    await queryDocs('rotp', 'items', { where: [['status', '==', 'doing']], limit: 5 });
    expect(lastCall()).toMatchObject({ path: '/api/p/rotp/db/items/query', method: 'POST' });
    expect(JSON.parse(String(lastCall().body))).toEqual({ where: [['status', '==', 'doing']], limit: 5 });

    reply({ id: 'O-1', status: 'applied', outcome: null, entry: null, seq: 2 });
    await setDoc('rotp', 'entries', 'O-1', { kind: 'ack' });
    expect(lastCall()).toMatchObject({ path: '/api/p/rotp/db/entries/O-1', method: 'PUT' });
    expect(JSON.parse(String(lastCall().body))).toEqual({ data: { kind: 'ack' } });

    reply({ results: [] });
    await batchDocs('rotp', [{ op: 'delete', coll: 'entries', id: 'O-1' }]);
    expect(lastCall()).toMatchObject({ path: '/api/p/rotp/db/batch', method: 'POST' });
  });

  it('encodes the scope, collection and id', async () => {
    reply({ seq: 1, id: 'a b', exists: false, data: null });
    await getDoc('_portfolio', 'cards', 'a b');
    expect(lastCall().path).toBe('/api/p/_portfolio/db/cards/a%20b');
  });

  it('lists the projects', async () => {
    reply([{ key: 'rotp', name: 'RotP', root: null, url: '/p/rotp/', hasBoard: true, running: 0 }]);
    expect((await listProjects())[0]?.key).toBe('rotp');
    expect(lastCall().path).toBe('/api/projects');
  });
});

describe('K2 api errors', { tags: ['critical'] }, () => {
  it('turns the error shape into an ApiError', async () => {
    reply({ error: { code: 'not_found', message: 'no items doc', details: { id: 'I-9' } } }, 404);
    const failure = await getDoc('rotp', 'items', 'I-9').catch((error: unknown) => error);
    expect(failure).toBeInstanceOf(ApiError);
    expect(failure).toMatchObject({ code: 'not_found', status: 404, message: 'no items doc', details: { id: 'I-9' } });
  });

  it('names a 401 without a body unauthorized', async () => {
    fetchMock.mockResolvedValueOnce(new Response('', { status: 401 }));
    await expect(listProjects()).rejects.toMatchObject({ code: 'unauthorized', status: 401 });
  });

  it('reports an unreachable server as unavailable', async () => {
    fetchMock.mockRejectedValueOnce(new TypeError('failed'));
    await expect(listProjects()).rejects.toMatchObject({ code: 'unavailable', status: 0 });
  });
});

describe('K2 api actions', { tags: ['critical'] }, () => {
  it('posts entries, results, proofs and sends down', async () => {
    reply({ id: 'O-1', status: 'applied', outcome: null, entry: null, seq: 3 });
    await postEntry('rotp', { id: 'O-1', kind: 'ack' });
    expect(lastCall()).toMatchObject({ path: '/api/p/rotp/entries', method: 'POST' });

    reply({ id: 'O-2', status: 'applied', outcome: null, entry: null, seq: 4 });
    await recordResult('rotp', 'T-1', 'pass', 'fine');
    expect(lastCall().path).toBe('/api/p/rotp/tests/T-1/result');
    expect(JSON.parse(String(lastCall().body))).toEqual({ outcome: 'pass', text: 'fine' });

    reply({ test: 'T-1', proof: { n: 1, url: '/files/rotp/proofs/a.png', caption: 'shot' } });
    await attachProof('rotp', 'T-1', new Blob(['x'], { type: 'image/png' }), 'shot');
    expect(lastCall()).toMatchObject({ path: '/api/p/rotp/tests/T-1/proofs?caption=shot', method: 'POST', headers: { 'Content-Type': 'image/png' } });

    reply({ id: 'O-3', status: 'applied', outcome: null, entry: null, seq: 5 });
    await sendDown({ project: 'all', text: 'hello', decision: false });
    expect(lastCall().path).toBe('/api/portfolio/send');
  });

  it('starts, stops, lists runs and sends stdin', async () => {
    reply({ run: { id: 'R-1' }, already: false });
    await startRun('rotp', { name: 'dev', visible: false });
    expect(lastCall()).toMatchObject({ path: '/api/p/rotp/runs', method: 'POST' });

    reply({ run: { id: 'R-1' } });
    await stopRun('rotp', 'R-1');
    expect(lastCall().path).toBe('/api/p/rotp/runs/R-1/stop');

    reply({ ok: true, bytes: 2 });
    await sendStdin('rotp', 'R-1', 'y\n');
    expect(lastCall().path).toBe('/api/p/rotp/runs/R-1/stdin');

    reply({ runs: [{ id: 'R-1' }] });
    expect(await listRuns(null, { status: 'running' })).toHaveLength(1);
    expect(lastCall().path).toBe('/api/runs?status=running');

    reply({ runs: [] });
    await listRuns('rotp', { limit: 3 });
    expect(lastCall().path).toBe('/api/p/rotp/runs?limit=3');
  });
});
