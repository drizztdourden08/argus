/* @layer renderer-app @kind types */
import type { KindCollection, KindFace } from './kind.type';

type ContentKind = 'line' | 'slug' | 'rich' | 'enum' | 'ref' | 'refs' | 'date' | 'person' | 'flag' | 'records';

type RuleMode = 'warn' | 'refuse';

interface LineSettings {
  markup: boolean;
  newlines: boolean;
  block: string;
}

interface PatternSettings {
  pattern: string;
}

interface ForbiddenSyntax {
  name: string;
  pattern: string;
}

interface RichSettings {
  rule: string;
  bullets: number;
  bulletMax: number;
  leadMax: number;
  shortMax: number;
  bullet: string;
  image: string;
  forbidden: ForbiddenSyntax[];
}

interface ContentSettings {
  line: LineSettings;
  slug: PatternSettings;
  rich: RichSettings;
  enum: Record<string, never>;
  ref: { kinds: Record<string, string> };
  refs: Record<string, never>;
  date: PatternSettings;
  person: PatternSettings;
  records: Record<string, never>;
}

interface ItemField {
  content: ContentKind;
  max?: number;
  required?: boolean;
  list?: number;
  images?: number;
  values?: string[];
  labels?: string[];
  to?: string[];
  leads?: boolean;
}

interface StageGate {
  need: string;
  mode: RuleMode;
}

interface ItemStage {
  id: string;
  label: string;
  lane: string;
  next?: string[];
  requires?: string[];
  gates?: StageGate[];
}

interface FieldGroup {
  id: string;
  label: string;
  fields: string[];
}

interface DoneRule {
  needs: string[];
  mode: RuleMode;
}

interface ItemType extends KindFace {
  id: string;
  label: string;
  icon: string;
  color?: string;
  tone?: string;
  collection: 'items' | 'tests' | 'launchers' | 'worktrees';
  idKey?: string;
  serves?: string[];
  readOnly?: boolean;
  hideOnKanban?: boolean;
  parents: string[];
  stages: ItemStage[];
  statuses?: string[];
  fields: string[];
  groups?: FieldGroup[];
  title: string;
  done?: DoneRule;
}

interface SubItemType extends KindFace {
  id: string;
  label: string;
  store: 'entries' | 'actions';
  icon: string;
  tone: string;
  blocksWhile?: 'open' | 'pending';
  field: string;
}

interface RecordType extends KindFace {
  id: string;
  label: string;
  stored?: string;
  shares?: string;
  aliases?: string[];
  icon: string;
  color?: string;
  tone?: string;
  collection: KindCollection;
  idKey?: string;
  title: string;
}

interface PersonLook {
  initials: string;
  color: string;
}

interface OwnerLook {
  label: string;
  initials: string;
  color: string;
}

interface PeopleLooks {
  owner: OwnerLook;
  leads: Record<string, PersonLook>;
}

interface ItemRegistry {
  version: number;
  lanes: string[];
  content: ContentSettings;
  baseFields: string[];
  fields: Record<string, ItemField>;
  types: ItemType[];
  subItems: SubItemType[];
  records: RecordType[];
  people: PeopleLooks;
  limits: Record<string, number>;
}

export type { ContentKind, ItemField, ItemRegistry, ItemStage, ItemType, RecordType, SubItemType };
