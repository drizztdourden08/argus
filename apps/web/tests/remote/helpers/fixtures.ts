/* @layer renderer-app @kind test */
import type { PullRequest } from '../../../src/data/types';

const MIN = 60 * 1000;

const ago = (ms: number): string => new Date(Date.now() - ms).toISOString();

const replay = (state: 'pending' | 'done' | 'failed', url: string | null = null) => ({ state, at: state === 'pending' ? null : ago(MIN), url });

const prDoc = (id: string, extra: object = {}) => ({
  id,
  title: `Title of ${id}`,
  body: '',
  head: 'feat/login',
  base: 'main',
  headCommit: 'abcdef1234567',
  status: 'open' as PullRequest['status'],
  item: 'I-12',
  checks: [] as string[],
  createdBy: 'lead',
  createdAt: ago(5 * MIN),
  mergedAt: null,
  mergeCommit: null,
  replay: replay('pending'),
  ...extra,
});

const stepDoc = (name: string, extra: object = {}) => ({ name, command: `pnpm ${name}`, exitCode: 0, durationMs: 1500, required: true, ...extra });

const versionDoc = (version: string, extra: object = {}) => ({
  version,
  tag: `v${version}`,
  commit: '0123456789abcdef',
  title: `Release ${version}`,
  publishedAt: ago(10 * MIN),
  notesUrl: `/releases/rotp/stable/${version}/notes.md`,
  assets: [{ name: `rotp-${version}-setup.exe`, size: 120 * 1024 * 1024, kind: 'installer', url: `/releases/rotp/stable/${version}/rotp-${version}-setup.exe` }],
  packages: [{ name: '@drizztdourden08/brock-core', version, size: 2048, url: `/releases/rotp/stable/${version}/brock-core-${version}.tgz` }],
  replay: replay('pending'),
  ...extra,
});

export { ago, MIN, prDoc, replay, stepDoc, versionDoc };
