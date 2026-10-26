/* @layer renderer-app @kind logic */
import { ApiError } from './api-error';
import { UNREACHABLE } from './api-request.constants';
import type { ApiErrorBody, RequestOptions } from './api.type';

const requestInit = (options: RequestOptions): RequestInit => {
  const method = options.method ?? 'GET';
  if (options.raw) return { method, headers: { 'Content-Type': options.rawType ?? options.raw.type }, body: options.raw };
  if (options.json !== undefined) return { method, headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(options.json) };
  const headers = method === 'GET' ? undefined : { 'Content-Type': 'application/json' };
  return { method, headers };
};

const parseBody = async (response: Response): Promise<unknown> => {
  const text = await response.text();
  if (!text) return null;
  try {
    return JSON.parse(text);
  } catch {
    return null;
  }
};

const errorBodyOf = (response: Response, body: unknown): ApiErrorBody => {
  const wrapped = (body as { error?: ApiErrorBody } | null)?.error;
  if (wrapped) return wrapped;
  const code = response.status === 401 ? 'unauthorized' : `http_${response.status}`;
  return { code, message: response.statusText || 'request failed' };
};

const apiRequest = async <T>(path: string, options: RequestOptions = {}): Promise<T> => {
  let response: Response;
  try {
    response = await fetch(path, requestInit(options));
  } catch {
    throw new ApiError(0, UNREACHABLE);
  }
  const body = await parseBody(response);
  if (!response.ok) throw new ApiError(response.status, errorBodyOf(response, body));
  return body as T;
};

export { apiRequest };
