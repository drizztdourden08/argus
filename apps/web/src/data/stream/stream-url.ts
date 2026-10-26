/* @layer renderer-app @kind logic */
import { scopePath } from '../api';

const streamUrl = (scope: string, collections: readonly string[], since: number | null): string => {
  const colls = collections.map(encodeURIComponent).join(',');
  const resume = since === null ? '' : `&since=${since}`;
  return `${scopePath(scope, 'stream')}?c=${colls}${resume}`;
};

export { streamUrl };
