import { useEffect } from "react";
import { listenForDuplicateEvents } from "../ipc/events";

export function useDuplicateEvents() {
  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listenForDuplicateEvents().then((cleanup) => {
      if (disposed) cleanup(); else unlisten = cleanup;
    }).catch(() => undefined);
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);
}
