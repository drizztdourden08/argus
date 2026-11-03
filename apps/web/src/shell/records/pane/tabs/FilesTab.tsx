/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { useCallback } from 'react';
import { readBoardReleases, useRemoteRead } from '../../../../data/remote';
import { EmptyLine } from '../../../../screens/board/tabs/kanban/sub-components/EmptyLine';
import { VersionFiles } from '../../../../screens/releases';
import type { RecordBodyProps } from '../pane.type';
import { FILES_READING, NO_FILES } from './FilesTab.constants';

const FilesTab = (props: RecordBodyProps) => {
  const { boardKey, target } = props.data;
  const load = useCallback(() => readBoardReleases(boardKey), [boardKey]);
  const { data } = useRemoteRead(load);
  if (!data) return <Text tone="muted">{FILES_READING}</Text>;
  const version = data.flatMap((channel) => channel.versions).find((candidate) => candidate.version === target.id);
  return version ? <VersionFiles version={version} /> : <EmptyLine>{NO_FILES}</EmptyLine>;
};

export { FilesTab };
