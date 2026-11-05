/* @layer renderer-app @kind logic */
import { TRAIL } from './trail.constants';
import type { TrailPoint } from './trail.type';

const weightOf = (age: number): number => Math.max(0, 1 - age / TRAIL.lifetimeSeconds) ** TRAIL.fadePower;

const trailToUniform = (trail: readonly TrailPoint[]): number[][] =>
  Array.from({ length: TRAIL.size }, (_, index) => {
    const point = trail[index];
    return point ? [point.x, 1 - point.y,weightOf(point.age)] : [0, 0, 0];
  });

export { trailToUniform };
