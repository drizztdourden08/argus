/* @layer renderer-app @kind logic */
import { apiRequest } from '../../../../data/api';
import { readRemote, useRemoteRead } from '../../../../data/remote';
import type { HealthInfo, SystemReads, VersionInfo } from './system.type';

const readHealth = (): Promise<HealthInfo> => apiRequest<HealthInfo>('/api/health');
const readVersion = (): Promise<VersionInfo> => apiRequest<VersionInfo>('/api/version');

const useSystemReads = (): SystemReads => ({
  health: useRemoteRead(readHealth).data,
  version: useRemoteRead(readVersion).data,
  remote: useRemoteRead(readRemote).data,
});

export { useSystemReads };
