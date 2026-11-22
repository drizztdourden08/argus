/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { RulesPage } from './overlord.type';

const readRules = (): Promise<RulesPage> => apiRequest<RulesPage>('/api/overlord/rules');

export { readRules };
