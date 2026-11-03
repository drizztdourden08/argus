/* @layer tooling-scripts @kind data */
const EMERALD = '#12a06e';

const FULL_EYES = [
  [14, 24, 22, 32],
  [42, 24, 50, 32],
];
const MEDIUM_EYES = [
  [7, 12, 11, 16],
  [21, 12, 25, 16],
];
const SMALL_EYES = [
  [4, 6, 6, 8],
  [10, 6, 12, 8],
];

const FULL_FACETS = [
  { points: [[4, 16], [36, 4], [32, 24]], tone: 0 },
  { points: [[36, 4], [44, 14], [32, 24]], tone: 1 },
  { points: [[36, 4], [60, 12], [60, 28], [44, 14]], accent: true },
  { points: [[4, 16], [32, 24], [32, 36], [4, 36]], tone: 2, eye: 0 },
  { points: [[44, 14], [60, 28], [60, 36], [32, 36], [32, 24]], tone: 3, eye: 1 },
  { points: [[4, 36], [32, 36], [18, 48]], tone: 4 },
  { points: [[18, 48], [32, 36], [32, 60]], tone: 5 },
  { points: [[32, 36], [60, 36], [46, 48]], tone: 6 },
  { points: [[32, 36], [46, 48], [32, 60]], tone: 7 },
];

const MEDIUM_FACETS = [
  { points: [[2, 8], [18, 2], [22, 7], [16, 12]], tone: 0 },
  { points: [[18, 2], [30, 6], [30, 14], [22, 7]], accent: true },
  { points: [[2, 8], [16, 12], [16, 18], [2, 18]], tone: 2, eye: 0 },
  { points: [[22, 7], [30, 14], [30, 18], [16, 18], [16, 12]], tone: 3, eye: 1 },
  { points: [[2, 18], [16, 18], [16, 30]], tone: 4 },
  { points: [[16, 18], [30, 18], [16, 30]], tone: 6 },
];

const SMALL_FACETS = [
  { points: [[1, 4], [9, 1], [11, 4], [8, 6], [8, 15], [1, 9]], tone: 0, eye: 0 },
  { points: [[9, 1], [15, 3], [15, 7], [11, 4]], accent: true },
  { points: [[11, 4], [15, 7], [15, 9], [8, 15], [8, 6]], tone: 5, eye: 1 },
];

const LEVELS = {
  full: { grid: 64, facets: FULL_FACETS, eyes: FULL_EYES },
  medium: { grid: 32, facets: MEDIUM_FACETS, eyes: MEDIUM_EYES },
  small: { grid: 16, facets: SMALL_FACETS, eyes: SMALL_EYES },
};

const TONES = {
  a: {
    accent: EMERALD,
    stone: ['#d6d6dc', '#c4c4cb', '#b8b8bf', '#a8a8b0', '#a2a2aa', '#8e8e96', '#8c8c94', '#767680'],
  },
  b: {
    accent: '#5fd6a6',
    stone: ['#3fcf9a', '#2dba88', '#25b080', '#1fa878', '#18a070', '#12906a', '#0e8760', '#0a6f4b'],
  },
};

export { EMERALD, LEVELS, TONES };
