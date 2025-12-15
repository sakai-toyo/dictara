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
    <div className="w-screen h-screen flex bg-gray-950 overflow-hidden shadow-[0_8px_32px_rgba(0,0,0,0.3)] font-sans">
      <div className="w-[60px] shrink-0 bg-yellow-400 flex items-center justify-center"></div>
      <div className="flex-1 bg-green-400 flex items-center justify-center text-white text-4xl"></div>
      <div className="w-[60px] shrink-0 bg-red-400 flex items-center justify-center"></div>
    </div>
  );
}

export default RecordingPopup;
