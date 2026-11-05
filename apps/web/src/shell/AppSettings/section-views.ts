/* @layer renderer-app @kind logic */
import type { ComponentType } from 'react';
import type { SectionId } from './AppSettings.type';
import { AppearanceSection } from './sections/AppearanceSection';
import { NotificationsSection } from './sections/NotificationsSection';
import { SystemSection } from './sections/SystemSection';
import { ProfileSection } from './sub-components/ProfileSection';

const SECTION_VIEWS: Record<SectionId, ComponentType> = {
  profile: ProfileSection,
  notifications: NotificationsSection,
  appearance: AppearanceSection,
  system: SystemSection,
};

export { SECTION_VIEWS };
