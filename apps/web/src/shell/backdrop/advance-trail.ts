/* @layer renderer-app @kind logic */
import type { PointerPoint } from './pointer-math.type';
import { TRAIL } from './trail.constants';
import type { TrailPoint } from './trail.type';

const advanceTrail = (trail: readonly TrailPoint[], seconds: number, sample: PointerPoint | null): TrailPoint[] => {
  const aged = trail
    .map((point) => ({ ...point, age: point.age + seconds }))
    .filter((point) => point.age < TRAIL.lifetimeSeconds);
  if (!sample) return aged;
  const [newest, ...older] = aged;
  if (newest && newest.age < TRAIL.sampleSeconds) return [{ ...newest, x: sample.x, y: sample.y }, ...older];
  return [{ x: sample.x, y: sample.y, age: 0 }, ...aged].slice(0, TRAIL.size);
};

export { advanceTrail };
