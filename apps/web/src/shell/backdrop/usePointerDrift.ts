/* @layer renderer-app @kind logic */
import { useEffect, useRef } from 'react';
import { MAX_FRAME_SECONDS } from './backdrop.constants';
import { BLOB } from './blob.constants';
import type { BlobState } from './blob.type';
import { easeBlob } from './ease-blob';
import { easePoint } from './ease-point';
import { isBlobSettled } from './is-blob-settled';
import { isSettled } from './is-settled';
import { mapPointer } from './map-pointer';
import { CENTER } from './pointer-math.constants';
import type { DriftFrame, PointerPoint } from './pointer-math.type';

const usePointerDrift = (enabled: boolean, onFrame: (frame: DriftFrame) => void): void => {
  const sink = useRef(onFrame);
  sink.current = onFrame;

  useEffect(() => {
    if (!enabled) {
      sink.current({ point: CENTER, blob: CENTER, blobWeight: 0 });
      return undefined;
    }
    let shown = CENTER;
    let target = CENTER;
    let blob: BlobState = { point: CENTER, presence: 0 };
    let present = false;
    let frame = 0;
    let last = 0;
    const step = (now: number): void => {
      const seconds = Math.min(MAX_FRAME_SECONDS, (now - last) / 1000);
      last = now;
      shown = isSettled(shown, target) ? target : easePoint(shown, target, seconds);
      blob = isBlobSettled(blob, target, present)
        ? { point: target, presence: present ? 1 : 0 }
        : easeBlob(blob, target, present, seconds);
      sink.current({ point: shown, blob: blob.point, blobWeight: BLOB.weight * blob.presence });
      frame = shown === target && isBlobSettled(blob, target, present) ? 0 : requestAnimationFrame(step);
    };
    const follow = (next: PointerPoint, inside: boolean): void => {
      target = next;
      if (inside && blob.presence === 0) blob = { point: next, presence: 0 };
      present = inside;
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
