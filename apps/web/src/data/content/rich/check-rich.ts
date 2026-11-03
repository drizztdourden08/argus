/* @layer renderer-app @kind logic */
import type { ItemField } from '../../registry';
import type { ContentContext, Verdict } from '../content.type';
import { classifyLine } from './classify-line';
import { refuseBullets } from './refuse-bullets';
import { refuseForbidden } from './refuse-forbidden';
import { refuseImages } from './refuse-images';
import { refuseLone } from './refuse-lone';
import { refuseLong } from './refuse-long';
import { refusePlain } from './refuse-plain';

const LINE_BREAK = /\r?\n/;

const checkRich = (field: string, spec: ItemField, text: string, cx: ContentContext): Verdict => {
  const raw = text
    .split(LINE_BREAK)
    .map((line) => line.trimEnd())
    .filter((line) => line.trim() !== '');
  const lines = raw.map(classifyLine);
  const refusal =
    refuseForbidden(field, raw) ??
    refuseLone(lines) ??
    refusePlain(field, lines) ??
    refuseBullets(field, lines) ??
    refuseImages(field, spec, lines, cx) ??
    refuseLong(field, spec, lines);
  return refusal ?? { ok: true, value: text.trim() };
};

export { checkRich };
