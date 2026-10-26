/* @layer renderer-app @kind constants */
import type { ApiErrorBody } from './api.type';

const UNREACHABLE: ApiErrorBody = { code: 'unavailable', message: 'dashboard unreachable' };

export { UNREACHABLE };
