/* @layer renderer-app @kind logic */
import { POINTER } from './backdrop.constants';
import type { BlobState } from './blob.type';
import { isSettled } from './is-settled';
import type { PointerPoint } from './pointer-math.type';

const isBlobSettled = (state: BlobState, target: PointerPoint, present: boolean): boolean =>
  isSettled(state.point, target) && Math.abs(state.presence - (present ? 1 : 0)) < POINTER.rest;

export { isBlobSettled };
