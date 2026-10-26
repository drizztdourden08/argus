/* @layer renderer-app @kind logic */
import type { LogRow } from '@drizztdourden08/tessera';
import { agoText } from '../../format';
import { isFresh } from '../../seen';
import { kindOf } from './kind-of';
import { levelOf } from './level-of';
import type { ActivityItem, RowContext } from './model.type';

const tagOf = (item: ActivityItem, context: RowContext): string => {
  const level = levelOf(item);
  const project = typeof item.project === 'string' && item.project ? context.nameOf(item.project) : '';
  const thread = typeof item.thread === 'string' && item.thread ? item.thread : '';
  const place = [project, thread].filter(Boolean).join(' / ');
  const fresh = isFresh(item.at, context.freshSince) ? 'new' : '';
  return [fresh, level === 'info' ? '' : level, place].filter(Boolean).join(' ');
};

const toLogRow = (item: ActivityItem, context: RowContext): LogRow => ({
  id: item.id,
  gutter: agoText(item.at, context.now),
  tag: tagOf(item, context),
  kind: kindOf(item),
  message: item.text ?? '',
});

export { toLogRow };
