/* @layer renderer-app @kind types */
type RefusalKey =
  | 'unknown-field'
  | 'required'
  | 'too-long'
  | 'newline'
  | 'line-markup'
  | 'not-text'
  | 'list-too-long'
  | 'not-a-list'
  | 'bad-slug'
  | 'bad-enum'
  | 'bad-date'
  | 'bad-ref'
  | 'unknown-ref'
  | 'bad-flag'
  | 'bad-person'
  | 'unknown-person'
  | 'bad-record'
  | 'rich-lone-paragraph'
  | 'rich-lead-late'
  | 'rich-lead-long'
  | 'rich-bullets-many'
  | 'rich-bullet-long'
  | 'rich-forbidden'
  | 'rich-image-unknown'
  | 'rich-image-many'
  | 'rich-text-long';

interface ContentContext {
  images?: readonly number[];
  ids?: readonly string[];
  threads?: readonly string[];
}

interface Accepted {
  ok: true;
  value: unknown;
}

interface Rejected {
  ok: false;
  key: RefusalKey;
  message: string;
}

type Verdict = Accepted | Rejected;

type RichLine = { kind: 'bullet'; text: string } | { kind: 'image'; n: number | null } | { kind: 'plain'; text: string };

export type { ContentContext, Rejected, RefusalKey, RichLine, Verdict };
