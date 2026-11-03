/* @layer renderer-app @kind logic */
import type { RegistryKind } from '../../../data/registry';
import { categoryColor } from './category-color';
import { NEUTRAL_COLOR, TONE_COLORS } from './kind-color.constants';

const kindColor = (kind: RegistryKind): string => {
  const category = categoryColor(kind.look.color);
  if (category) return `var(--c-tag-${category})`;
  return (kind.look.tone ? TONE_COLORS[kind.look.tone] : undefined) ?? NEUTRAL_COLOR;
};

export { kindColor };
