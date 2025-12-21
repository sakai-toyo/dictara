import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";

interface FnKeyEvent {
  pressed: boolean;
  timestamp: number;
}

interface ListenerError {
  error: string;
  is_permission_error: boolean;
}

interface RecordingStoppedEvent {
  text: string;
}

function App() {
  const [fnPressed, setFnPressed] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [hasPermission, setHasPermission] = useState(true);
  const [checkingPermission, setCheckingPermission] = useState(true);
  const listenerStartedRef = useRef(false);

  // Transcription state
  const [transcribing, setTranscribing] = useState(false);
  const [transcriptionText, setTranscriptionText] = useState<string>("");
  const [transcriptionError, setTranscriptionError] = useState<string | null>(null);

  // Check permission on mount
  useEffect(() => {
    checkPermission();
  }, []);

  // Listen for FN key events
  useEffect(() => {
    const unlisten = listen<FnKeyEvent>("fn-key-event", (event) => {
      setFnPressed(event.payload.pressed);
      setError(null); // Clear error if we're receiving events
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  // Listen for listener errors
  useEffect(() => {
    const unlisten = listen<ListenerError>("fn-listener-error", (event) => {
      setError(event.payload.error);
      if (event.payload.is_permission_error) {
        setHasPermission(false);
      }
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  // Listen for recording started event
  useEffect(() => {
    const unlisten = listen("recording-started", () => {
      console.log("[Recording] Started");
      setTranscribing(true);
      setTranscriptionError(null);
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  // Listen for recording stopped event
  useEffect(() => {
    const unlisten = listen<RecordingStoppedEvent>("recording-stopped", (event) => {
      console.log("[Recording] Stopped:", event.payload);
      setTranscribing(false);
      setTranscriptionText(event.payload.text);
      setTranscriptionError(null);
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  // Start listener once permission is granted and the app is ready
  useEffect(() => {
    if (checkingPermission || !hasPermission || listenerStartedRef.current) return;

    listenerStartedRef.current = true;
    // invoke("start_fn_listener").catch((err) => {
    //   listenerStartedRef.current = false; // allow retry if it fails
    //   const message = err instanceof Error ? err.message : String(err);
    //   setError(message);
    // });
  }, [checkingPermission, hasPermission]);

  async function checkPermission() {
    setCheckingPermission(true);
    const permitted = await invoke<boolean>("check_accessibility_permission");
    setHasPermission(permitted);
    setCheckingPermission(false);

    if (!permitted) {
      setError("Accessibility permission required");
    }
  }

  async function requestPermission() {
    await invoke("request_accessibility_permission");
    // Wait a bit for user to potentially grant permission
    setTimeout(checkPermission, 1000);
  }

  async function restartApp() {
    await invoke("restart_app");
  }

  if (checkingPermission) {
    return (
      <main className="container mx-auto flex flex-col items-center justify-center min-h-screen p-4">
        <h1 className="text-3xl font-bold mb-4">Dictara - FN Key Monitor</h1>
        <p className="text-muted-foreground">Checking permissions...</p>
      </main>
    );
  }

  if (!hasPermission) {
    return (
      <main className="container mx-auto flex flex-col items-center justify-center min-h-screen p-4">
        <h1 className="text-3xl font-bold mb-8">Dictara - FN Key Monitor</h1>

        <Alert variant="destructive" className="max-w-2xl">
          <AlertTitle className="text-xl mb-4">⚠️ Permission Required</AlertTitle>
          <AlertDescription className="space-y-4">
            <p>This app needs Accessibility permission to monitor keyboard events.</p>

            <div>
              <h3 className="font-semibold mb-2">Setup Instructions:</h3>
              <ol className="list-decimal list-inside space-y-1">
                <li>Click "Open System Settings" below</li>
                <li>In Privacy & Security → Accessibility</li>
                <li>Find "Dictara" in the list</li>
                <li>Toggle the switch ON</li>
                <li>Click "Restart App" below</li>
              </ol>
            </div>

            <div className="flex gap-3 pt-2">
              <Button onClick={requestPermission}>
                Open System Settings
              </Button>
              <Button onClick={restartApp} variant="secondary">
                Restart App
              </Button>
            </div>
          </AlertDescription>
        </Alert>
      </main>
    );
  }

  return (
    <main className="container mx-auto flex flex-col items-center min-h-screen p-8">
      <h1 className="text-3xl font-bold mb-8">Dictara - FN Key Monitor</h1>

      {error && (
        <Alert variant="destructive" className="mb-6 max-w-2xl">
          <AlertTitle>Error</AlertTitle>
          <AlertDescription className="space-y-3">
            <p>{error}</p>
            <Button onClick={restartApp} variant="outline" size="sm">
              Restart App
            </Button>
          </AlertDescription>
        </Alert>
      )}

      <div className="my-12">
        <Badge
          variant={fnPressed ? "default" : "destructive"}
          className="text-2xl px-20 py-10 transition-all duration-200 ease-in-out"
          style={{
            backgroundColor: fnPressed ? "#4CAF50" : "#f44336",
            boxShadow: fnPressed ? "0 8px 16px rgba(76, 175, 80, 0.4)" : "0 4px 8px rgba(0,0,0,0.2)"
          }}
        >
          FN Pressed: {fnPressed ? "True" : "False"}
        </Badge>
      </div>

      <p className="text-sm text-muted-foreground mb-8">
        Press and hold the FN key on your keyboard to test
      </p>

      {/* Transcription Section */}
      <div className="w-full max-w-2xl space-y-4">
        <h2 className="text-2xl font-semibold">Transcription</h2>

        {transcribing && (
          <Card className="bg-blue-500 text-white border-blue-600">
            <CardContent className="pt-6 text-center">
              <div className="text-lg font-bold">🎙️ Transcribing audio...</div>
              <div className="text-sm mt-2 opacity-90">Please wait</div>
            </CardContent>
          </Card>
        )}

        {transcriptionError && (
          <Alert variant="destructive">
            <AlertTitle>❌ Transcription Error</AlertTitle>
            <AlertDescription>{transcriptionError}</AlertDescription>
          </Alert>
        )}

        {!transcribing && !transcriptionError && transcriptionText && (
          <Card className="border-green-500">
            <CardHeader>
              <div className="flex justify-between items-center">
                <CardTitle className="text-green-600">✅ Transcription Result</CardTitle>
                <Button
                  onClick={() => {
                    navigator.clipboard.writeText(transcriptionText);
                  }}
                  size="sm"
                  variant="outline"
                >
                  📋 Copy
                </Button>
              </div>
            </CardHeader>
            <CardContent>
              <p className="text-base leading-relaxed whitespace-pre-wrap break-words">
                {transcriptionText}
              </p>
            </CardContent>
          </Card>
        )}

        {!transcribing && !transcriptionError && !transcriptionText && (
          <Card className="bg-muted">
            <CardContent className="pt-6 text-center text-muted-foreground">
              <p>No transcription yet. Hold FN key to record and transcribe.</p>
            </CardContent>
          </Card>
        )}
      </div>
    </main>
  );
}

export default App;
