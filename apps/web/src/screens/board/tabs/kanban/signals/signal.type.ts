/* @layer renderer-app @kind types */
import type { BrandApp } from '@drizztdourden08/tessera/brand';
import type { AvatarProps, IconName, SignalTone } from '@drizztdourden08/tessera';

type SignalOpens = { kind: 'item' } | { kind: 'entry'; id: string };

interface CardSignal {
  id: string;
  icon: IconName;
  count?: number | string;
  label: string;
  tone: SignalTone;
  opens: SignalOpens;
  avatar?: AvatarProps;
  brand?: BrandApp;
}

interface SignalOpener {
  item: () => void;
  entry: (id: string) => void;
}

export type { CardSignal, SignalOpener, SignalOpens };
