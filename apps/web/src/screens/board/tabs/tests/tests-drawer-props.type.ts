/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { WithId } from '../../../../data/react';
import type { TestItem, TestResult } from '../../../../data/types';
import type { ItemDoc, TestDoc } from './model';
import type { TestOps, TestsData, TestsScreen } from './tests.type';

interface TestFactsProps {
  test: TestDoc;
  handed: boolean;
  ops: TestOps;
  items: ItemDoc[];
}

interface ResultBlockProps {
  test: TestDoc;
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
  runs: TestResult[];
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
  AcListProps, DrawerSectionProps, EditDialogProps, EvidenceProps, NoteDialogProps, ProofUploadFormProps, ResultBlockProps, ResultHistoryProps, ShotPreviewProps, TestFactsProps, TestLayersProps,
};
