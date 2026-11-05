/* @layer renderer-app @kind logic */
import { CENTER } from './pointer-math.constants';
import type { PointerPoint } from './pointer-math.type';

const clampUnit = (value: number): number => Math.min(1, Math.max(0, value));

const mapPointer = (clientX: number, clientY: number, width: number, height: number): PointerPoint => {
  if (width <= 0 || height <= 0) return CENTER;
  return { x: clampUnit(clientX / width), y: clampUnit(clientY / height) };
};

export { mapPointer };
