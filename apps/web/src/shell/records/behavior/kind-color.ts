/* @layer renderer-app @kind logic */
import type { RegistryKind } from '../../../data/registry';
import { NEUTRAL_COLOR, TONE_COLORS } from './kind-color.constants';

const kindColor = (kind: RegistryKind): string => (kind.look.tone ? TONE_COLORS[kind.look.tone] : undefined) ?? NEUTRAL_COLOR;

export { kindColor };
