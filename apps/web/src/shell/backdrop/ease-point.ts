/* @layer renderer-app @kind logic */
import { easeStep } from './ease-step';
import type { PointerPoint } from './pointer-math.type';

const easePoint = (current: PointerPoint, target: PointerPoint, seconds: number): PointerPoint => ({
  x: easeStep(current.x, target.x, seconds),
  y: easeStep(current.y, target.y, seconds),
});

export { easePoint };
