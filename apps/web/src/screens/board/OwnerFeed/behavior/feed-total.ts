/* @layer renderer-app @kind logic */
import type { FeedSectionsProps } from '../OwnerFeed.type';

const feedTotal = (work: FeedSectionsProps['work']): number => work.waiting.length + work.leads.length + work.notices.length + work.lines.length;

export { feedTotal };
