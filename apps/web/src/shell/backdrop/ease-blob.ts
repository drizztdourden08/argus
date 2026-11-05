/* @layer renderer-app @kind logic */
import { BLOB } from './blob.constants';
import type { BlobState } from './blob.type';
import { easeStep } from './ease-step';
import type { PointerPoint } from './pointer-math.type';

const easeBlob = (state: BlobState, target: PointerPoint, present: boolean, seconds: number): BlobState => {
  const follow = BLOB.followSeconds * BLOB.settleFactor;
  return {
    point: {
      x: easeStep(state.point.x, target.x, seconds, follow),
      y: easeStep(state.point.y, target.y, seconds, follow),
    },
    presence: easeStep(state.presence, present ? 1 : 0, seconds, BLOB.fadeSeconds * BLOB.settleFactor),
  };
};

export { easeBlob };
