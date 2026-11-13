/* @layer renderer-app @kind types */
import type { LogLine } from '../../../../data/types';
import type { FeedCard } from '../../feed-model';
import type { FeedContext, FeedFilter } from '../OwnerFeed.type';

interface LogWindow {
  shown: number;
  left: number;
  more: () => void;
  less: () => void;
}

interface LogListProps {
  lines: LogLine[];
}

interface LogSectionProps {
  lines: LogLine[];
}

interface LogFoldProps {
  filter: FeedFilter;
  notices: FeedCard[];
  lines: LogLine[];
  ctx: FeedContext;
}

export type { LogFoldProps, LogListProps, LogSectionProps, LogWindow };
