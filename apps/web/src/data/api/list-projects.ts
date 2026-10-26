/* @layer renderer-app @kind logic */
import { apiRequest } from './api-request';
import type { ProjectSummary } from './api.type';

const listProjects = (): Promise<ProjectSummary[]> => apiRequest<ProjectSummary[]>('/api/projects');

export { listProjects };
