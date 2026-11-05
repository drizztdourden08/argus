/* @layer renderer-app @kind logic */
import { useEffect, useRef } from 'react';
import { MAX_FRAME_SECONDS } from './backdrop.constants';
import { easePoint } from './ease-point';
import { isSettled } from './is-settled';
import { mapPointer } from './map-pointer';
import { CENTER } from './pointer-math.constants';
import type { PointerPoint } from './pointer-math.type';

const usePointerDrift = (enabled: boolean, onPoint: (point: PointerPoint) => void): void => {
  const sink = useRef(onPoint);
  sink.current = onPoint;

  useEffect(() => {
    if (!enabled) {
      sink.current(CENTER);
      return undefined;
    }
    let shown = CENTER;
    let target = CENTER;
    let frame = 0;
    let last = 0;
    const step = (now: number): void => {
      const seconds = Math.min(MAX_FRAME_SECONDS, (now - last) / 1000);
      last = now;
      shown = isSettled(shown, target) ? target : easePoint(shown, target, seconds);
      sink.current(shown);
      frame = shown === target ? 0 : requestAnimationFrame(step);
    };
    const follow = (next: PointerPoint): void => {
      target = next;
      if (frame === 0) {
        last = performance.now();
        frame = requestAnimationFrame(step);
      }
    };
    const onMove = (event: PointerEvent): void =>
      follow(mapPointer(event.clientX, event.clientY, window.innerWidth, window.innerHeight));
    const onLeave = (): void => follow(CENTER);
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
