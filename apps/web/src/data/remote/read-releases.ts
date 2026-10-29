/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { ProjectReleases } from './remote.type';

const readReleases = async (): Promise<ProjectReleases[]> => (await apiRequest<{ projects: ProjectReleases[] }>('/api/releases')).projects;

export { readReleases };
