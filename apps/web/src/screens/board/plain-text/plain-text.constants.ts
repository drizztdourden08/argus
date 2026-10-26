/* @layer renderer-app @kind constants */
const MAX_CHARS = 20000;

const MAX_LINE_CHARS = 2000;

const FENCE_PATTERN = /^ {0,3}(`{3,}|~{3,})/;

const RULE_PATTERN = /^ {0,3}([-*_])(?: *\1){2,} *$/;

const TABLE_RULE_PATTERN = /^[ |:-]+$/;

const BLOCK_MARKS: readonly (readonly [RegExp, string])[] = [
  [/^ {0,3}#{1,6}(?: +|$)/, ''],
  [/^ *(?:> ?)+/, ''],
  [/^ *(?:[-*+]|\d{1,9}[.)]) +(?:\[[ xX]\] +)?/, ''],
];

const INLINE_MARKS: readonly (readonly [RegExp, string])[] = [
  [/!\[([^\]\n]{0,200})\]\([^)\n]{0,2048}\)/g, '$1'],
  [/\[([^\]\n]{0,300})\]\([^)\n]{0,2048}\)/g, '$1'],
  [/(`+)([^`\n]{1,500}?)\1/g, '$2'],
  [/(^|[\s(>])(\*\*|__)(?=\S)([^\n]{0,300}?)(?<=\S)\2(?=$|[\s).,;:!?])/g, '$1$3'],
  [/(^|[\s(>])([*_])(?=\S)([^\n*_]{1,300}?)(?<=\S)\2(?=$|[\s).,;:!?])/g, '$1$3'],
  [/~~([^~\n]{1,300})~~/g, '$1'],
  [/\\([\\`*_{}[\]()#+\-.!~|>])/g, '$1'],
];

export { BLOCK_MARKS, FENCE_PATTERN, INLINE_MARKS, MAX_CHARS, MAX_LINE_CHARS, RULE_PATTERN, TABLE_RULE_PATTERN };
