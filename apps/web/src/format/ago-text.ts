/* @layer renderer-app @kind logic */
import { formatAgo } from './format-ago';
import { toMillis } from './to-millis';

const agoText = (value: unknown, now: number): string =>
  Number.isFinite(toMillis(value)) ? `${formatAgo(value, now)} ago` : '';

export { agoText };
