import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import "./RecordingPopup.css";

function RecordingPopup() {
  useEffect(() => {
    const unlistenStart = listen("recording-started", () => {
      console.log("[Popup] Recording started");
    });

    const unlistenStop = listen("recording-stopped", () => {
      console.log("[Popup] Recording stopped");
    });

    return () => {
      unlistenStart.then((f) => f());
      unlistenStop.then((f) => f());
    };
  }, []);

  return (
    <div className="w-screen h-screen flex items-center justify-center bg-transparent overflow-hidden rounded-full font-sans">
      <div className="w-full h-full bg-red-500 rounded-full flex items-center justify-center shadow-[0_8px_32px_rgba(0,0,0,0.3)]">
      </div>
    </div>
  );
}

export default RecordingPopup;
