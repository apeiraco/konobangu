import {
  type Dispatch,
  type RefObject,
  type SetStateAction,
  useCallback,
  useRef,
  useState,
} from "react";

export function useStateRef<T>(
  initialValue: T | (() => T),
): [T, Dispatch<SetStateAction<T>>, RefObject<T>] {
  const [state, _setState] = useState(initialValue);
  const ref = useRef(state);

  const setState = useCallback((value: T | ((prev: T) => T)) => {
    let nextValue: T;
    if (typeof value === "function") {
      nextValue = (value as (prev: T) => T)(ref.current);
    } else {
      nextValue = value;
    }
    ref.current = nextValue;
    _setState(nextValue);
  }, []);

  return [state, setState, ref] as const;
}
