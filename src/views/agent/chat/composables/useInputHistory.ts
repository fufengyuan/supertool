import { ref } from 'vue';
import type { Ref } from 'vue';

interface UseInputHistoryArgs {
  currentInput: Ref<string>;
  applyText: (text: string) => void;
}

interface UseInputHistoryResult {
  push: (text: string) => void;
  recallPrev: () => boolean;
  recallNext: () => boolean;
  isNavigating: () => boolean;
  size: () => number;
}

/**
 * Terminal-style command history navigation.
 * State is held in refs so that key presses don't trigger Vue re-renders
 * for navigation alone — only the applied text change re-renders the textarea.
 */
export function useInputHistory({
  currentInput,
  applyText,
}: UseInputHistoryArgs): UseInputHistoryResult {
  const history = ref<string[]>([]);
  const indexRef = { value: -1 };
  const draftRef = { value: '' };

  const push = (text: string) => {
    history.value = [...history.value, text];
    indexRef.value = -1;
    draftRef.value = '';
  };

  const recallPrev = (): boolean => {
    if (history.value.length === 0) return false;
    const cur = indexRef.value;
    const next = cur === -1 ? history.value.length - 1 : Math.max(0, cur - 1);
    if (cur === -1) draftRef.value = currentInput.value;
    indexRef.value = next;
    applyText(history.value[next]);
    return true;
  };

  const recallNext = (): boolean => {
    const cur = indexRef.value;
    if (cur === -1) return false;
    if (cur < history.value.length - 1) {
      indexRef.value = cur + 1;
      applyText(history.value[cur + 1]);
    } else {
      indexRef.value = -1;
      applyText(draftRef.value);
    }
    return true;
  };

  const isNavigating = () => indexRef.value !== -1;
  const size = () => history.value.length;

  return { push, recallPrev, recallNext, isNavigating, size };
}
