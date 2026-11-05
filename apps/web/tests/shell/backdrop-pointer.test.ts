/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { MOTION, STILL_MOTION } from '../../src/shell/backdrop/backdrop.constants';
import { motionFor } from '../../src/shell/backdrop/motion-rule';
import { CENTER, easePoint, easeStep, isSettled, mapPointer } from '../../src/shell/backdrop/pointer-math';

describe('backdrop pointer easing', { tags: ['critical'] }, () => {
  it('converges toward the target and never overshoots', () => {
    let value = 0;
    for (let frame = 0; frame < 400; frame += 1) {
      const next = easeStep(value, 1, 1 / 60);
      expect(next).toBeGreaterThanOrEqual(value);
      expect(next).toBeLessThanOrEqual(1);
      value = next;
    }
    expect(value).toBeGreaterThan(0.999);
  });

  it('settles in about two seconds', () => {
    let value = 0;
    for (let frame = 0; frame < 108; frame += 1) value = easeStep(value, 1, 1 / 60);
    expect(value).toBeGreaterThan(0.94);
    expect(value).toBeLessThan(0.99);
  });

  it('does not overshoot on a long frame and ignores a zero step', () => {
    expect(easeStep(0, 1, 5)).toBeLessThanOrEqual(1);
    expect(easeStep(0.3, 1, 0)).toBe(0.3);
    expect(easePoint({ x: 1, y: 0 }, { x: 0, y: 1 }, 10).x).toBeGreaterThanOrEqual(0);
  });

  it('reports a settled point', () => {
    expect(isSettled(CENTER, { x: 0.5004, y: 0.5 })).toBe(true);
    expect(isSettled(CENTER, { x: 0.6, y: 0.5 })).toBe(false);
  });
});

describe('backdrop pointer mapping', { tags: ['critical'] }, () => {
  it('maps client pixels to the unit square and clamps', () => {
    expect(mapPointer(960, 540, 1920, 1080)).toEqual({ x: 0.5, y: 0.5 });
    expect(mapPointer(-20, 2000, 1920, 1080)).toEqual({ x: 0, y: 1 });
  });

  it('falls back to the centre for an empty window', () => {
    expect(mapPointer(10, 10, 0, 0)).toEqual(CENTER);
  });
});

describe('backdrop motion rule', { tags: ['critical'] }, () => {
  it('stands still with reduced motion or a hidden page', () => {
    expect(motionFor(true, true)).toBe(STILL_MOTION);
    expect(motionFor(false, false)).toBe(STILL_MOTION);
    expect(STILL_MOTION.speed).toBe(0);
    expect(STILL_MOTION.interactive).toBe(false);
  });

  it('moves and follows the pointer otherwise', () => {
    expect(motionFor(false, true)).toBe(MOTION);
    expect(MOTION.interactive).toBe(true);
  });
});
