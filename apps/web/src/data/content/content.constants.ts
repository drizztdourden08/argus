/* @layer renderer-app @kind constants */
import { ITEM_REGISTRY } from '../registry';
import { patternOf } from './pattern-of';

const KINDS = ITEM_REGISTRY.content;

const LINE_BLOCK = patternOf(KINDS.line.block);

const SLUG_PATTERN = patternOf(KINDS.slug.pattern);

const DATE_PATTERN = patternOf(KINDS.date.pattern);

const PERSON_PATTERN = patternOf(KINDS.person.pattern);

const REF_PATTERNS: ReadonlyMap<string, RegExp> = new Map(Object.entries(KINDS.ref.kinds).map(([kind, source]): [string, RegExp] => [kind, patternOf(source)]));

const RICH_RULE = KINDS.rich;

const RICH_BULLET = patternOf(KINDS.rich.bullet);

const RICH_IMAGE = patternOf(KINDS.rich.image);

const RICH_FORBIDDEN = KINDS.rich.forbidden.map((banned) => ({ name: banned.name, pattern: patternOf(banned.pattern) }));

const LONE_PARAGRAPH =
  'Write it as bullets (- point) or add a screenshot; a lone paragraph is not accepted. Example: Short lead line, then - first point and - second point, one per line.';

const MAX_IMAGE_NUMBER = 4_294_967_295;

const NEWLINE = /[\n\r]/;

const LEAD_PREFIX = 'lead:';

const MONTH_DAYS = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

export { DATE_PATTERN, LEAD_PREFIX, LINE_BLOCK, LONE_PARAGRAPH, MAX_IMAGE_NUMBER, MONTH_DAYS, NEWLINE, PERSON_PATTERN, REF_PATTERNS, RICH_BULLET, RICH_FORBIDDEN, RICH_IMAGE, RICH_RULE, SLUG_PATTERN };
