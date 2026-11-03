/* @layer renderer-app @kind test */
import type { CheckRun, ReleaseChannel, ReleaseVersion } from '../../src/data/remote';
import type { PullRequest } from '../../src/data/types';
import type { PullRequestDoc } from '../../src/screens/releases/releases.type';

const WHEN = '2026-10-26T12:00:00.000Z';

const prOf = (id: string, extra: Partial<PullRequest> = {}): PullRequestDoc => ({
  id,
  title: `Title of ${id}`,
  head: 'feat/login',
  base: 'main',
  status: 'open',
  item: 'I-12',
  createdAt: WHEN,
  mergedAt: null,
  mergeCommit: null,
  replay: { state: 'pending', at: null, url: null },
  ...extra,
});

const checkRunOf = (id: string, extra: Partial<CheckRun> = {}): CheckRun => ({
  id,
  runId: `run-${id}`,
  project: 'rotp',
  branch: 'feat/login',
  commit: 'abcdef1234567',
  pr: 'PR-1',
  trigger: 'pr',
  status: 'pass',
  startedAt: WHEN,
  finishedAt: WHEN,
  steps: [
    { name: 'lint', command: 'pnpm lint', exitCode: 0, durationMs: 1500, required: true },
    { name: 'test', command: 'pnpm test', exitCode: 0, durationMs: 65_000, required: true },
  ],
  runner: { host: 'box', os: 'win32', node: '24.0.0', pnpm: '10.0.0' },
  summary: 'All required steps passed',
  ...extra,
});

const versionOf = (version: string, extra: Partial<ReleaseVersion> = {}): ReleaseVersion => ({
  version,
  tag: `v${version}`,
  commit: '0123456789abcdef',
  title: `Release ${version}`,
  publishedAt: WHEN,
  notesUrl: null,
  assets: [{ name: `rotp-${version}-setup.exe`, size: 1024, kind: 'installer', url: `/releases/rotp/stable/${version}/setup.exe` }],
  packages: [{ name: '@drizztdourden08/brock-core', version, size: 2048, url: `/releases/rotp/stable/${version}/brock-core.tgz` }],
  replay: null,
  ...extra,
});

const channelOf = (channel: string, latest: string, versions: ReleaseVersion[]): ReleaseChannel => ({ channel, latest, versions });

export { channelOf, checkRunOf, prOf, versionOf };
