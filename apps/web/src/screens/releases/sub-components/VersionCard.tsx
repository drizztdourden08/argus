/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { RecordCard, recordMenuGroups, useRecordPane } from '../../../shell';
import { assigneeOfRecord } from '../../../shell/records';
import { kindOf } from '../../../data/registry';
import { copyText } from '../../board/shared';
import { releaseCode } from '../model';
import { RELEASE_KIND } from '../releases.constants';
import type { VersionCardProps } from '../releases.type';
import { ShippedRows } from './ShippedRows';
import { VersionFoot } from './VersionFoot';

const VersionCard = (props: VersionCardProps) => {
  const { channel, version, latest, project, prefixed, shipped, layout } = props;
  const pane = useRecordPane();
  const code = releaseCode(project, version.version, prefixed);
  const kind = kindOf(RELEASE_KIND);
  const title = version.title;
  const open = () => pane.open(code);
  return (
    <RecordCard
      head={{
        kind: RELEASE_KIND,
        code,
        assignee: kind ? assigneeOfRecord(kind, {}, project) : '',
        boardKey: project,
        threads: [],
        menu: recordMenuGroups(RELEASE_KIND, code, { open, copy: () => copyText(code) }),
        menuLabel: `More actions for ${code}`,
      }}
      title={title}
      time={version.publishedAt}
      compact={layout === 'compact'}
      openLabel={`Open ${code}: ${title}`}
      onOpen={open}
      label={`Release ${version.version} of channel ${channel}`}
      edge="border"
      edgeTone={latest ? 'success' : undefined}
      attrs={{ 'data-version': version.version }}
      foot={(
        <Flex direction="column" gap="xs">
          <VersionFoot version={version} layout={layout} />
          {shipped && layout !== 'compact' && <ShippedRows version={version} project={project} source={shipped} />}
        </Flex>
      )}
    />
  );
};

export { VersionCard };
