/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { WithId } from '../../../../data/react';
import type { TestItem } from '../../../../data/types';
import type { TestDoc } from './model';
import type { DrawerNav, DrawerWide, NoticeOps, TestsData, TestsScreen } from './tests.type';

interface TestDrawerProps {
  test: TestDoc | null;
  item: WithId<TestItem> | null;
  handed: boolean;
  nav: DrawerNav;
  wide: DrawerWide;
  notices: NoticeOps;
  onClose: () => void;
}

interface TestToolsProps {
  test: TestDoc;
  nav: DrawerNav;
  wide: DrawerWide;
}

interface TestFactsProps {
  test: TestDoc;
  handed: boolean;
}

interface ResultBlockProps {
  test: TestDoc;
}

interface TestBodyProps {
  test: TestDoc;
  item: WithId<TestItem> | null;
  handed: boolean;
  notices: NoticeOps;
}

interface AcListProps {
  test: TestDoc;
  item: WithId<TestItem> | null;
}

interface EvidenceProps {
  test: TestDoc;
  boardKey: string;
}

interface ProofUploadFormProps {
  testId: string;
  boardKey: string;
}

interface ResultHistoryProps {
  runs: TestDoc['runs'];
}

interface ShotPreviewProps {
  file: File;
  testId: string;
  onRemove: () => void;
}

interface DrawerSectionProps {
  title: string;
  count?: number;
  children: ReactNode;
}

interface TextListProps {
  lines: string[];
  ordered?: boolean;
  label: string;
}

interface StepListProps {
  lines: string[];
}

interface NoteDialogProps {
  test: TestDoc;
  onClose: () => void;
}

interface EditDialogProps {
  test: TestDoc;
  onClose: () => void;
}

interface TestLayersProps {
  screen: TestsScreen;
  data: TestsData;
  boardKey: string;
}

export type {
  AcListProps, DrawerSectionProps, EditDialogProps, EvidenceProps, NoteDialogProps, ProofUploadFormProps, ResultBlockProps, ResultHistoryProps, ShotPreviewProps, StepListProps, TestBodyProps, TestDrawerProps, TestFactsProps, TestLayersProps, TestToolsProps, TextListProps,
};
