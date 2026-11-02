/* @layer renderer-app @kind logic */
import { depColor } from '../../../../../data/deps';
import type { ItemRef } from '../../../../../data/kanban';
import { depLinks } from './dep-links';
import { depTagLabel } from './dep-tag-label';
import { depTagTitle } from './dep-tag-title';
import type { DepLink, DepTagSpec } from './model.type';

const tagOf = (link: DepLink, now: number): DepTagSpec => ({
  key: link.side,
  color: link.side === 'asked' ? 'neutral' : depColor(link.state),
  label: depTagLabel(link),
  title: depTagTitle(link, now),
});

const depTags = (item: ItemRef, now: number): DepTagSpec[] => depLinks(item).map((link) => tagOf(link, now));

export { depTags };
