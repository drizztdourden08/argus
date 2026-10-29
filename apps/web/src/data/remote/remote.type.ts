/* @layer renderer-app @kind types */
type RemoteMode = 'local' | 'remote';

type ReplayState = 'pending' | 'done' | 'failed';

interface Replay {
  state: ReplayState;
  at: string | null;
  url: string | null;
}

interface ReplayCounts {
  prs: number;
  releases: number;
  packages: number;
  checks: number;
}

interface RemoteInfo {
  mode: RemoteMode;
  replay: Partial<ReplayCounts>;
  ciRoots?: Readonly<Record<string, string>>;
}

interface ReleaseAsset {
  name: string;
  size: number;
  kind: string;
  url: string;
}

interface ReleasePackage {
  name: string;
  version: string;
  size: number;
  url: string;
}

interface ReleaseVersion {
  version: string;
  tag: string;
  commit: string;
  title: string;
  publishedAt: string;
  notesUrl: string | null;
  assets: ReleaseAsset[];
  packages: ReleasePackage[];
  replay: Replay | null;
}

interface ReleaseChannel {
  channel: string;
  latest: string | null;
  versions: ReleaseVersion[];
}

interface ProjectReleases {
  project: string;
  channels: ReleaseChannel[];
}

type CheckStatus = 'queued' | 'running' | 'pass' | 'fail' | 'cancelled' | 'skipped-local';

interface CheckStep {
  name: string;
  command: string;
  status?: CheckStatus;
  exitCode: number | null;
  durationMs: number | null;
  required?: boolean;
}

interface CheckRunner {
  host: string;
  os: string;
  node: string;
  pnpm: string;
}

interface CheckRun {
  id: string;
  runId: string;
  project: string;
  branch: string;
  commit: string;
  pr: string | null;
  trigger: string;
  status: CheckStatus;
  startedAt: string | null;
  finishedAt: string | null;
  steps: CheckStep[];
  runner: CheckRunner | null;
  summary: string;
}

interface CheckQuery {
  commit?: string;
  pr?: string;
}

interface RegistryBuild {
  name: string;
  version: string;
  commit: string;
  checkout: string;
  packedAt: string;
  tarball: string;
  size: number;
}

interface RegistryConsumer {
  path: string;
  packages: string[];
  on: boolean;
}

interface RegistryReport {
  builds: RegistryBuild[];
  consumers: RegistryConsumer[];
  scopeRedirected: boolean;
}

interface RemoteRead<T> {
  data: T | null;
  error: string;
  reload: () => void;
}

export type {
  CheckQuery, CheckRun, CheckStatus, CheckStep, ProjectReleases, RegistryBuild, RegistryConsumer, RegistryReport, ReleaseChannel,
  ReleaseVersion, RemoteInfo, RemoteRead, Replay, ReplayCounts, ReplayState,
};
