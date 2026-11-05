/* @layer renderer-app @kind logic */
import { POINTER } from './backdrop.constants';

type PointerPoint = { readonly x: number; readonly y: number };

const CENTER: PointerPoint = { x: POINTER.center, y: POINTER.center };

const clampUnit = (value: number): number => Math.min(1, Math.max(0, value));

const mapPointer = (clientX: number, clientY: number, width: number, height: number): PointerPoint => {
  if (width <= 0 || height <= 0) return CENTER;
  return { x: clampUnit(clientX / width), y: clampUnit(clientY / height) };
};

const easeStep = (current: number, target: number, seconds: number): number => {
  if (seconds <= 0) return current;
  const follow = 1 - Math.exp((-3 * seconds) / POINTER.settleSeconds);
  return current + (target - current) * follow;
};

const easePoint = (current: PointerPoint, target: PointerPoint, seconds: number): PointerPoint => ({
  x: easeStep(current.x, target.x, seconds),
  y: easeStep(current.y, target.y, seconds),
});

const isSettled = (current: PointerPoint, target: PointerPoint): boolean =>
  Math.abs(current.x - target.x) < POINTER.rest && Math.abs(current.y - target.y) < POINTER.rest;

export { CENTER, easePoint, easeStep, isSettled, mapPointer };
export type { PointerPoint };
