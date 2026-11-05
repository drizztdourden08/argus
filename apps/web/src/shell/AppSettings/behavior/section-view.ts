/* @layer renderer-app @kind logic */
import type { ComponentType } from 'react';
import type { SectionId } from '../AppSettings.type';
import { AppearanceSection } from '../sub-components/AppearanceSection';
import { NotificationsSection } from '../sub-components/NotificationsSection';
import { ProfileSection } from '../sub-components/ProfileSection';
import { SystemSection } from '../sub-components/SystemSection';

const sectionView = (id: SectionId): ComponentType => {
  const views: Record<SectionId, ComponentType> = {
    profile: ProfileSection,
    notifications: NotificationsSection,
    appearance: AppearanceSection,
    system: SystemSection,
  };
  return views[id];
};

export { sectionView };
