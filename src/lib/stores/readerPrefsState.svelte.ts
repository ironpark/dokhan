import type { ReaderFontSize, ReaderLineHeight, ReaderWidth } from '$lib/types/dictionary';

export type ReaderPrefsSnapshot = {
  preprocessEnabled: boolean;
  markerPreprocessEnabled: boolean;
  readerFontSize: ReaderFontSize;
  readerLineHeight: ReaderLineHeight;
  readerWidth: ReaderWidth;
};

export function createReaderPrefsState(onChange: () => void) {
  let preprocessEnabled = $state(true);
  let markerPreprocessEnabled = $state(true);
  let readerFontSize = $state<ReaderFontSize>('md');
  let readerLineHeight = $state<ReaderLineHeight>('normal');
  let readerWidth = $state<ReaderWidth>('normal');

  return {
    get preprocessEnabled() {
      return preprocessEnabled;
    },
    get markerPreprocessEnabled() {
      return markerPreprocessEnabled;
    },
    get readerFontSize() {
      return readerFontSize;
    },
    get readerLineHeight() {
      return readerLineHeight;
    },
    get readerWidth() {
      return readerWidth;
    },
    applySnapshot(snapshot: ReaderPrefsSnapshot) {
      preprocessEnabled = snapshot.preprocessEnabled;
      markerPreprocessEnabled = snapshot.markerPreprocessEnabled;
      readerFontSize = snapshot.readerFontSize;
      readerLineHeight = snapshot.readerLineHeight;
      readerWidth = snapshot.readerWidth;
    },
    toSnapshot(): ReaderPrefsSnapshot {
      return {
        preprocessEnabled,
        markerPreprocessEnabled,
        readerFontSize,
        readerLineHeight,
        readerWidth
      };
    },
    setPreprocessEnabled(enabled: boolean) {
      preprocessEnabled = enabled;
      onChange();
    },
    setMarkerPreprocessEnabled(enabled: boolean) {
      markerPreprocessEnabled = enabled;
      onChange();
    },
    setReaderFontSize(value: ReaderFontSize) {
      readerFontSize = value;
      onChange();
    },
    setReaderLineHeight(value: ReaderLineHeight) {
      readerLineHeight = value;
      onChange();
    },
    setReaderWidth(value: ReaderWidth) {
      readerWidth = value;
      onChange();
    }
  };
}

export type ReaderPrefsState = ReturnType<typeof createReaderPrefsState>;
