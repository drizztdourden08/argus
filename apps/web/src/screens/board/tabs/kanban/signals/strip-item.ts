/* @layer renderer-app @kind logic */
import type { SignalItem } from '@drizztdourden08/tessera';
import { OWNER_DEFAULTS } from '../../../../../owner';
import type { OwnerProfile } from '../../../../../owner';
import { ownerMark } from './owner-mark';
import type { CardSignal, SignalOpener } from './signal.type';

const stripItem = (signal: CardSignal, opener: SignalOpener, owner: OwnerProfile = OWNER_DEFAULTS): SignalItem => {
  const { id, icon, label, tone, opens, count, avatar, brand } = signal;
  const onClick = () => (opens.kind === 'entry' ? opener.entry(opens.id) : opener.item());
  return {
    id,
    icon,
    label,
    tone,
    onClick,
    ...(count === undefined ? {} : { count }),
    ...(brand === undefined ? {} : { mark2: { brand } }),
    ...(brand !== undefined || avatar === undefined ? {} : { mark2: avatar.variant === 'owner' ? ownerMark(owner) : { avatar } }),
  };
};

export { stripItem };
