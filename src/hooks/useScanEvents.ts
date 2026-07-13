import { useEffect } from "react";
import { listenForScanEvents } from "../ipc/events";

export function useScanEvents() {
  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listenForScanEvents().then((cleanup) => {
      if (disposed) cleanup(); else unlisten = cleanup;
    }).catch(() => undefined);
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);
}

