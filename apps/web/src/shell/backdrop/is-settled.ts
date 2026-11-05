/* @layer renderer-app @kind logic */
import { POINTER } from './backdrop.constants';
import type { PointerPoint } from './pointer-math.type';

const isSettled = (current: PointerPoint, target: PointerPoint): boolean =>
  Math.abs(current.x - target.x) < POINTER.rest && Math.abs(current.y - target.y) < POINTER.rest;

export { isSettled };
