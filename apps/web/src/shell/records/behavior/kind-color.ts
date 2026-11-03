/* @layer renderer-app @kind logic */
import type { TypeMarkTone } from '@drizztdourden08/tessera';
import type { RegistryKind } from '../../../data/registry';
import type { KindMark } from '../records.type';
import { categoryColor } from './category-color';
import { MARK_TONES } from './kind-color.constants';

const isMarkTone = (tone: string | undefined): tone is TypeMarkTone => tone !== undefined && MARK_TONES.includes(tone);

const kindMark = (kind: RegistryKind): KindMark => {
  const color = categoryColor(kind.look.color);
  if (color) return { color };
  const { tone } = kind.look;
  return isMarkTone(tone) ? { tone } : {};
};

export { kindMark };
