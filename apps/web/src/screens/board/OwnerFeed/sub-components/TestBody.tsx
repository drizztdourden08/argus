/* @layer renderer-app @kind component */
import { Flex, Link, Tag, Text } from '@drizztdourden08/tessera';
import { ProofGallery } from '../../ProofGallery';
import type { TestBodyProps } from '../OwnerFeed.type';

const TestBody = (props: TestBodyProps) => {
  const { build, summary, title, proofs } = props;
  return (
    <Flex direction="column" gap="xs">
      <Flex gap="sm" align="center" wrap>
        {build ? <Tag>{build}</Tag> : null}
        <Text title={title}>{summary}</Text>
        <Link href="#tests">Open in Tests</Link>
      </Flex>
      <ProofGallery proofs={proofs ?? []} label={`Proofs of ${title}`} />
    </Flex>
  );
};

export { TestBody };
