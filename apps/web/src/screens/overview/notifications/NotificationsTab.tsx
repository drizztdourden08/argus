/* @layer renderer-app @kind component */
import { EmptyState, Stack } from '@drizztdourden08/tessera';
import { useMemo, useState } from 'react';
import { useNotificationFeed } from './behavior/useNotificationFeed';
import { useNotifySettings } from './behavior/useNotifySettings';
import { useTestPing } from './behavior/useTestPing';
import { countLevels, filterNotes, projectCounts } from './model';
import type { NoteFilters } from './model';
import { NO_FILTERS, NONE_YET } from './notifications.constants';
import type { NotificationsTabProps } from './notifications.type';
import { NoteFeed } from './sub-components/NoteFeed';
import { NoteFilterBar } from './sub-components/NoteFilterBar';
import { NotifySettings } from './sub-components/NotifySettings';
import { StallList } from './sub-components/StallList';
import { TestPingBar } from './sub-components/TestPingBar';

const NotificationsTab = (props: NotificationsTabProps) => {
  const { freshSince } = props;
  const notes = useNotificationFeed();
  const ping = useTestPing();
  const settings = useNotifySettings();
  const [filters, setFilters] = useState<NoteFilters>(NO_FILTERS);
  const shown = useMemo(() => filterNotes(notes, filters), [notes, filters]);
  const counts = useMemo(() => countLevels(notes), [notes]);
  const projects = useMemo(() => projectCounts(notes), [notes]);
  return (
    <Stack gap="md">
      <TestPingBar ping={ping} />
      <NotifySettings state={settings} />
      <StallList />
      {notes.length === 0
        ? <EmptyState message={NONE_YET} />
        : (
          <>
            <NoteFilterBar filters={filters} counts={counts} projects={projects} onChange={setFilters} />
            <NoteFeed notes={shown} freshSince={freshSince} />
          </>
        )}
    </Stack>
  );
};

export { NotificationsTab };
