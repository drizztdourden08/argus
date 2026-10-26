/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { agoText, clipText, formatAgo, formatCpu, formatMemory, formatMinutes, orText } from '../../src/screens/overview/format';

const NOW = Date.parse('2026-10-26T12:00:00.000Z');
const before = (ms: number): string => new Date(NOW - ms).toISOString();
const SECOND = 1000;
const MINUTE = 60 * SECOND;
const HOUR = 60 * MINUTE;

describe('formatAgo', () => {
  it('says seconds, minutes, hours and days, rounded', () => {
    expect(formatAgo(before(30 * SECOND), NOW)).toBe('30s');
    expect(formatAgo(before(5 * MINUTE), NOW)).toBe('5m');
    expect(formatAgo(before(3 * HOUR), NOW)).toBe('3h');
    expect(formatAgo(before(47 * HOUR), NOW)).toBe('47h');
    expect(formatAgo(before(50 * HOUR), NOW)).toBe('2d');
  });

  it('says never for a missing or unreadable time, and 0s for a time ahead of the clock', () => {
    expect(formatAgo(undefined, NOW)).toBe('never');
    expect(formatAgo(null, NOW)).toBe('never');
    expect(formatAgo('not a date', NOW)).toBe('never');
    expect(formatAgo(before(-5 * MINUTE), NOW)).toBe('0s');
  });

  it('takes epoch milliseconds and ISO text', () => {
    expect(formatAgo(NOW - 2 * MINUTE, NOW)).toBe('2m');
  });
});

describe('agoText', () => {
  it('adds ago, and says nothing for a time that is not one', () => {
    expect(agoText(before(5 * MINUTE), NOW)).toBe('5m ago');
    expect(agoText(undefined, NOW)).toBe('');
  });
});

describe('formatMemory', () => {
  it('shows MB under a gigabyte, one decimal of GB up to ten, whole GB after', () => {
    expect(formatMemory(512)).toBe('512 MB');
    expect(formatMemory(1536)).toBe('1.5 GB');
    expect(formatMemory(20480)).toBe('20 GB');
  });

  it('shows a dash for what is not a number', () => {
    expect(formatMemory(null)).toBe('-');
    expect(formatMemory(undefined)).toBe('-');
    expect(formatMemory('12')).toBe('-');
  });
});

describe('formatMinutes', () => {
  it('shows under a minute, minutes, hours with minutes, whole hours and days', () => {
    expect(formatMinutes(0.4)).toBe('<1m');
    expect(formatMinutes(45)).toBe('45m');
    expect(formatMinutes(125)).toBe('2h 5m');
    expect(formatMinutes(120)).toBe('2h');
    expect(formatMinutes(3000)).toBe('2d');
    expect(formatMinutes(null)).toBe('-');
  });
});

describe('formatCpu', () => {
  it('keeps one decimal under ten percent and rounds above', () => {
    expect(formatCpu(3.14)).toBe('3.1%');
    expect(formatCpu(42.6)).toBe('43%');
    expect(formatCpu(null)).toBe('-');
  });
});

describe('clipText and orText', () => {
  it('clips a long text and leaves a short one', () => {
    expect(clipText('abcdef', 4)).toBe('abc...');
    expect(clipText('abc', 4)).toBe('abc');
  });

  it('falls back on an empty or missing text', () => {
    expect(orText('', 'x')).toBe('x');
    expect(orText(null, 'x')).toBe('x');
    expect(orText('y', 'x')).toBe('y');
  });
});
