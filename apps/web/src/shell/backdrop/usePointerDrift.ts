/* @layer renderer-app @kind logic */
import { useEffect, useRef } from 'react';
import { MAX_FRAME_SECONDS } from './backdrop.constants';
import { advanceTrail } from './advance-trail';
import { easePoint } from './ease-point';
import { isSettled } from './is-settled';
import { mapPointer } from './map-pointer';
import { CENTER } from './pointer-math.constants';
import type { DriftFrame, PointerPoint } from './pointer-math.type';
import type { TrailPoint } from './trail.type';
import { trailToUniform } from './trail-to-uniform';

const usePointerDrift = (enabled: boolean, onFrame: (frame: DriftFrame) => void): void => {
  const sink = useRef(onFrame);
  sink.current = onFrame;

  useEffect(() => {
    if (!enabled) {
      sink.current({ point: CENTER, trail: trailToUniform([]) });
      return undefined;
    }
    let shown = CENTER;
    let target = CENTER;
    let trail: TrailPoint[] = [];
    let fresh = false;
    let frame = 0;
    let last = 0;
    const step = (now: number): void => {
      const seconds = Math.min(MAX_FRAME_SECONDS, (now - last) / 1000);
      last = now;
      shown = isSettled(shown, target) ? target : easePoint(shown, target, seconds);
      trail = advanceTrail(trail, seconds, fresh ? target : null);
      fresh = false;
      sink.current({ point: shown, trail: trailToUniform(trail) });
      frame = shown === target && trail.length === 0 ? 0 : requestAnimationFrame(step);
    };
    const follow = (next: PointerPoint, moved: boolean): void => {
      target = next;
      fresh = moved;
      if (frame === 0) {
        last = performance.now();
        frame = requestAnimationFrame(step);
      }
    };
    const onMove = (event: PointerEvent): void =>
      follow(mapPointer(event.clientX, event.clientY, window.innerWidth, window.innerHeight), true);
    const onLeave = (): void => follow(CENTER, false);
    window.addEventListener('pointermove', onMove, { passive: true });
    document.documentElement.addEventListener('pointerleave', onLeave, { passive: true });
    return () => {
      window.removeEventListener('pointermove', onMove);
      document.documentElement.removeEventListener('pointerleave', onLeave);
      cancelAnimationFrame(frame);
    };
  }, [enabled]);
};

export { usePointerDrift };
