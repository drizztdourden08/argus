/* @layer renderer-app @kind types */
import type { BrandApp } from '@drizztdourden08/tessera/brand';
import type { ReactNode } from 'react';

type SectionId = 'profile' | 'notifications' | 'appearance' | 'accounts' | 'system';

type AvatarMode = 'default' | 'initials' | 'brand' | 'upload';

interface AppSettingsState {
  open: boolean;
  section: SectionId;
}

interface AppSettingsApi extends AppSettingsState {
  openSettings: (section?: SectionId) => void;
  close: () => void;
  setSection: (section: SectionId) => void;
}

interface AppSettingsProviderProps {
  children: ReactNode;
}

interface AppSettingsDialogProps {
  section: SectionId;
  onSection: (section: SectionId) => void;
  onClose: () => void;
}

interface ProfileChoice {
  name: string;
  initials: string;
  avatar: string | null;
  brand: BrandApp | null;
  mode: AvatarMode;
  busy: boolean;
  problem: string | null;
  chooseMode: (mode: AvatarMode) => Promise<void>;
  pickBrand: (app: BrandApp) => Promise<void>;
  upload: (files: File[]) => Promise<void>;
  reject: () => void;
  reset: () => Promise<void>;
  saveName: (value: string) => Promise<void>;
  saveInitials: (value: string) => Promise<void>;
}

interface ProfileTextProps {
  value: string;
  label: string;
  maxLength?: number;
  disabled: boolean;
  onSave: (value: string) => void;
}

export type {
  AppSettingsApi,
  AppSettingsDialogProps,
  AppSettingsProviderProps,
  AppSettingsState,
  AvatarMode,
  ProfileChoice,
  ProfileTextProps,
  SectionId,
};
