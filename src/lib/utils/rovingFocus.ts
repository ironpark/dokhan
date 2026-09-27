export type RovingOrientation = 'horizontal' | 'vertical';

/**
 * Resolves the next index for arrow/Home/End navigation within a wrapping list,
 * or null when the key is not a navigation key for the given orientation.
 */
export function nextRovingIndex(
  key: string,
  index: number,
  count: number,
  orientation: RovingOrientation
): number | null {
  if (count <= 0) return null;
  const [prevKey, nextKey] =
    orientation === 'horizontal' ? ['ArrowLeft', 'ArrowRight'] : ['ArrowUp', 'ArrowDown'];
  switch (key) {
    case nextKey:
      return (index + 1) % count;
    case prevKey:
      return (index - 1 + count) % count;
    case 'Home':
      return 0;
    case 'End':
      return count - 1;
    default:
      return null;
  }
}
