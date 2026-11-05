/* @layer renderer-app @kind types */
import type { RemoteInfo } from '../../data/remote';

interface LocalBannerState {
  info: RemoteInfo;
  text: string;
  boardKey?: string;
}

interface LocalBannerProps {
  state: LocalBannerState;
}

export type { LocalBannerProps, LocalBannerState };
