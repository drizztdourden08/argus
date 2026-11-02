/* @layer renderer-app @kind logic */
import { agoText } from '../../../../../format';
import type { DepLink } from './model.type';

const depTagTitle = (link: DepLink, now: number): string =>
  [link.title, link.reason ? `Reason: ${link.reason}` : '', agoText(link.since, now)].filter(Boolean).join('. ');

export { depTagTitle };
