/* @layer renderer-app @kind component */
import { DropdownMenu } from '@drizztdourden08/tessera';
import type { MenuItem } from '@drizztdourden08/tessera';
import { copyText } from '../../shared';
import { threadState } from '../behavior/thread-state';
import type { ThreadMenuProps } from '../ThreadDrawer.type';

const liveItems = (props: ThreadMenuProps): MenuItem[] => {
  const { thread, paused, ops } = props;
  if (!threadState(thread).live) return [];
  return [{ id: 'pause', label: paused ? 'Resume' : 'Pause', onSelect: () => ops.setPaused(thread, !paused) }];
};

const archiveItems = (props: ThreadMenuProps): MenuItem[] => {
  const { thread, ops } = props;
  const state = threadState(thread);
  if (state.coordinator && !state.archived) return [];
  return [{ id: 'archive', label: state.archived ? 'Unarchive' : 'Archive', onSelect: () => ops.soft(state.archived ? 'unarchive' : 'archive', thread) }];
};

const purgeItems = (props: ThreadMenuProps): MenuItem[] => {
  const { thread, onPurge } = props;
  return threadState(thread).coordinator ? [] : [{ id: 'purge', label: 'Purge...', tone: 'danger', onSelect: () => onPurge(thread.id) }];
};

const subjectItems = (props: ThreadMenuProps): MenuItem[] => {
  const { onEditSubject } = props;
  return onEditSubject ? [{ id: 'subject', label: 'Edit subject', onSelect: onEditSubject }] : [];
};

const ThreadMenu = (props: ThreadMenuProps) => {
  const { thread } = props;
  const label = `More actions for ${thread.id}`;
  return (
    <DropdownMenu
      label={label}
      trigger={{ label, icon: 'ellipsis-vertical', iconOnly: true }}
      groups={[
        {
          id: 'thread',
          label: thread.id,
          items: [
            ...subjectItems(props),
            ...liveItems(props),
            ...archiveItems(props),
            ...purgeItems(props),
            { id: 'copy', label: 'Copy id', onSelect: () => copyText(thread.id) },
          ],
        },
      ]}
      variant="ghost"
      size="sm"
      align="end"
    />
  );
};

export { ThreadMenu };
