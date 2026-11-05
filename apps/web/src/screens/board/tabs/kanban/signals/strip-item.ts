/* @layer renderer-app @kind logic */
import type { SignalItem } from '@drizztdourden08/tessera';
import type { CardSignal, SignalOpener } from './signal.type';

const stripItem = (signal: CardSignal, opener: SignalOpener): SignalItem => {
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
    ...(brand !== undefined || avatar === undefined ? {} : { mark2: { avatar } }),
  };
};

export { stripItem };
