/* @layer renderer-app @kind logic */
import type { WithId } from '../../../../data/react';
import type { Wake } from '../../../../data/types';

const stamp = (wake: Wake): number => Date.parse(wake.at ?? '') || 0;

const passiveNotices = (wakes: readonly WithId<Wake>[]): WithId<Wake>[] =>
  wakes.filter((wake) => wake.status === 'passive').sort((a, b) => stamp(b) - stamp(a));

export { passiveNotices };
