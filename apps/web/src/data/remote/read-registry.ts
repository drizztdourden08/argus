/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { RegistryReport } from './remote.type';

const readRegistry = (): Promise<RegistryReport> => apiRequest<RegistryReport>('/api/registry');

export { readRegistry };
