/* @layer renderer-app @kind component */
import { Card, Flex } from '@drizztdourden08/tessera';
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
  const { channel, version, latest, project, prefixed, shipped } = props;
  const pane = useRecordPane();
  const code = releaseCode(project, version.version, prefixed);
  const kind = kindOf(RELEASE_KIND);
  const title = version.title || `Release ${version.version}`;
  const open = () => pane.open(code);
  return (
    <Card data-version={version.version} role="group" aria-label={`Release ${version.version} of channel ${channel}`} edge="border" edgeTone={latest ? 'success' : undefined}>
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
        openLabel={`Open ${code}: ${title}`}
        onOpen={open}
        foot={(
          <Flex direction="column" gap="xs">
            <VersionFoot version={version} latest={latest} />
            {shipped && <ShippedRows version={version} project={project} source={shipped} />}
          </Flex>
        )}
      />
    </Card>
  );
};

export { VersionCard };
