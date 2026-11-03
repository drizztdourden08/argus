/* @layer tooling-scripts @kind logic */
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { LEVELS, TONES } from './argus-facets.mjs';

const root = join(dirname(fileURLToPath(import.meta.url)), '..', '..');
const logoDir = join(root, 'apps', 'web', 'assets', 'logo');

const scaleOf = (level, size) => size / LEVELS[level].grid;

const snapX = (level, size, x) => {
  const { grid } = LEVELS[level];
  const scale = scaleOf(level, size);
  const half = grid / 2;
  if (x === half) return size / 2;
  return x < half ? Math.round(x * scale) : size - Math.round((grid - x) * scale);
};

const snapY = (level, size, y) => Math.round(y * scaleOf(level, size));

const snapPoint = (level, size) => ([x, y]) => [snapX(level, size, x), snapY(level, size, y)];

const eyeRect = (level, size, [x0, y0, x1, y1]) => {
  const snap = snapPoint(level, size);
  const [ax, ay] = snap([x0, y0]);
  const [bx, by] = snap([x1, y1]);
  return [ax, ay, bx, by];
};

const ring = (points) => `M${points.map(([x, y]) => `${x} ${y}`).join('L')}Z`;

const rectRing = ([x0, y0, x1, y1]) => ring([[x0, y0], [x1, y0], [x1, y1], [x0, y1]]);

const inside = ([px, py], points) => {
  let hit = false;
  points.forEach(([ax, ay], index) => {
    const [bx, by] = points[(index + 1) % points.length];
    if ((ay > py) !== (by > py) && px < ((bx - ax) * (py - ay)) / (by - ay) + ax) hit = !hit;
  });
  return hit;
};

const rectCorners = ([x0, y0, x1, y1]) => [[x0, y0], [x1, y0], [x1, y1], [x0, y1]];

const assertHole = (facet, rect, label) => {
  const strictlyIn = rectCorners(rect).every((corner) => inside(corner, facet.points));
  if (!strictlyIn) throw new Error(`eye leaves its facet at ${label}`);
};

const facetShapes = (level, size) => {
  const snap = snapPoint(level, size);
  const eyes = LEVELS[level].eyes.map((eye) => eyeRect(level, size, eye));
  return LEVELS[level].facets.map((facet) => {
    const points = facet.points.map(snap);
    const hole = facet.eye === undefined ? undefined : eyes[facet.eye];
    if (hole) assertHole({ points }, hole, `${level} ${size}`);
    return { ...facet, points, hole };
  });
};

const fillOf = (variant, facet) => (facet.accent ? TONES[variant].accent : TONES[variant].stone[facet.tone]);

const outlineOf = (shapes) => {
  const edges = new Map();
  shapes.forEach(({ points }) => {
    points.forEach((from, index) => {
      const to = points[(index + 1) % points.length];
      const key = `${from}>${to}`;
      const back = `${to}>${from}`;
      if (edges.has(back)) edges.delete(back);
      else edges.set(key, [from, to]);
    });
  });
  return [...edges.values()];
};

const chain = (edges) => {
  const next = new Map(edges.map(([from, to]) => [String(from), to]));
  const start = edges[0][0];
  const loop = [start];
  let at = next.get(String(start));
  while (String(at) !== String(start)) {
    loop.push(at);
    at = next.get(String(at));
  }
  return loop;
};

const basePath = (shapes) => {
  const outline = chain(outlineOf(shapes));
  const holes = shapes.filter(({ hole }) => hole).map(({ hole }) => rectRing(hole));
  return `${ring(outline)}${holes.join('')}`;
};

const facetPath = ({ points, hole }) => `${ring(points)}${hole ? rectRing(hole) : ''}`;

const element = (path, fill) => `  <path d="${path}" fill="${fill}" fill-rule="evenodd"/>`;

const logoSvg = (variant, level, size = LEVELS[level].grid) => {
  const shapes = facetShapes(level, size);
  const crisp = level === 'small';
  const attributes = crisp ? ' shape-rendering="crispEdges"' : '';
  const under = crisp ? [] : [element(basePath(shapes), fillOf(variant, shapes[shapes.length >> 1]))];
  const body = shapes.map((shape) => element(facetPath(shape), fillOf(variant, shape)));
  const name = variant === 'a' ? 'stone with emerald corner' : 'emerald';
  return [
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${size} ${size}" width="${size}" height="${size}"${attributes} role="img">`,
    `  <title>Argus, ${name}, ${level} detail</title>`,
    ...under,
    ...body,
    '</svg>',
    '',
  ].join('\n');
};

const writeLogos = () => {
  mkdirSync(logoDir, { recursive: true });
  for (const variant of ['a', 'b']) {
    for (const level of Object.keys(LEVELS)) {
      writeFileSync(join(logoDir, `argus-${variant}-${level}.svg`), logoSvg(variant, level));
    }
  }
};

if (process.argv[1] === fileURLToPath(import.meta.url)) writeLogos();

export { logoSvg, writeLogos };
