/* @layer renderer-app @kind types */
import type { AvatarProps, IconName, SignalTone } from '@drizztdourden08/tessera';

type SignalOpens = { kind: 'item' } | { kind: 'entry'; id: string };

interface CardSignal {
  id: string;
  icon: IconName;
  count?: number;
  label: string;
  tone: SignalTone;
  opens: SignalOpens;
  avatar?: AvatarProps;
}

interface SignalOpener {
  item: () => void;
  entry: (id: string) => void;
}

export type { CardSignal, SignalOpener, SignalOpens };
