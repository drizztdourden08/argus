/* @layer renderer-app @kind component */
import { Disclosure, Flex } from '@drizztdourden08/tessera';
import type { DependencySectionProps } from '../drawer-props.type';
import { depLinks } from '../model';
import { DependencyView } from './DependencyView';

const DependencySection = (props: DependencySectionProps) => {
  const { item, now } = props;
  const links = depLinks(item);
  if (links.length === 0) return null;
  return (
    <Disclosure summary="Dependency" defaultOpen size="sm">
      <Flex direction="column" gap="md">
        {links.map((link) => <DependencyView key={`${link.side}-${link.dep}`} link={link} now={now} />)}
      </Flex>
    </Disclosure>
  );
};

export { DependencySection };
