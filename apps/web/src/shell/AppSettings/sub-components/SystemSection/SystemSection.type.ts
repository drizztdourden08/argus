/* @layer renderer-app @kind types */
import type { RemoteInfo } from '../../../../data/remote';

interface HealthInfo {
  version?: string;
  startedAt?: string | null;
  port?: number;
  shadow?: boolean;
  preview?: boolean;
}

interface VersionInfo {
  version?: string;
  schema?: number;
}

interface SystemReads {
  health: HealthInfo | null;
  version: VersionInfo | null;
  remote: RemoteInfo | null;
}

interface SystemFact {
  label: string;
  value: string;
}

export type { HealthInfo, SystemFact, SystemReads, VersionInfo };
