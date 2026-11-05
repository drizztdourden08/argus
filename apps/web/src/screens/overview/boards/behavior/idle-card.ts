/* @layer renderer-app @kind logic */
import type { Card } from '../../../../data/types';
import type { BoardRecord } from '../model';

const idleCard = (record: BoardRecord): Card => ({ key: record.key, name: record.name, state: 'not started', root: null, url: `/p/${record.key}/` });

export { idleCard };
