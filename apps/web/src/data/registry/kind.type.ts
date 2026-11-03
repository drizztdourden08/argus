/* @layer renderer-app @kind types */
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

export type { CodeEntry, KindCollection, KindFace, RegistryKind };
