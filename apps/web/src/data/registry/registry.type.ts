/* @layer renderer-app @kind types */
type ContentKind = 'line' | 'slug' | 'rich' | 'enum' | 'ref' | 'refs' | 'date' | 'person' | 'flag' | 'records';

type RuleMode = 'warn' | 'refuse';

type CardProgress = 'ac' | 'children' | 'children-ac' | 'run' | 'checks' | 'none';

type KindView = 'card' | 'row';

type KindCollection =
  | 'items'
  | 'tests'
  | 'launchers'
  | 'worktrees'
  | 'agents'
  | 'prs'
  | 'releases'
  | 'deps'
  | 'runs'
  | 'checks'
  | 'entries'
  | 'actions';

type KindFamily = 'type' | 'subItem' | 'record';

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

interface ItemTypeCard {
  progress: CardProgress;
  chips: string[];
}

interface KindLink {
  field: string;
  to: string[];
  label: string;
  inverse?: string;
}

interface KindFace {
  code: string;
  view: KindView;
  assignee: string;
  tabs: string[];
  card: ItemTypeCard;
  links: KindLink[];
  signals?: string[];
  groupBy?: string;
  action?: string;
}

interface ItemType extends KindFace {
  id: string;
  label: string;
  icon: string;
  color: string;
  collection: 'items' | 'tests' | 'launchers' | 'worktrees';
  idKey?: string;
  serves?: string[];
  readOnly?: boolean;
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

interface KindLook {
  color?: string;
  tone?: string;
}

interface RegistryKind {
  family: KindFamily;
  id: string;
  label: string;
  code: string;
  shares?: string;
  aliases: string[];
  stored?: string;
  icon: string;
  look: KindLook;
  collection: KindCollection;
  idKey?: string;
  view: KindView;
  title: string;
  assignee: string;
  tabs: string[];
  card: ItemTypeCard;
  links: KindLink[];
  signals: string[];
  groupBy?: string;
  action?: string;
}

interface CodeEntry {
  letters: string;
  kind: string;
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

export type {
  CardProgress,
  CodeEntry,
  ContentKind,
  ItemField,
  ItemRegistry,
  ItemStage,
  ItemType,
  KindCollection,
  KindFamily,
  KindLink,
  KindLook,
  KindView,
  RecordType,
  RegistryKind,
  SubItemType,
};
