// notes to self:
// useState and useEffect are hooks that add behavior to a component without writing a class
// useState gives a component memory
// useEffect lets a component run code when events happen
import "./App.css";
import { useState, useEffect } from "react";
// invoke functions in the Rust backend over Tauri's IPC bridge
import { invoke } from "@tauri-apps/api/core";
// listen to events that Rust emits
import { listen } from "@tauri-apps/api/event";
import { Button } from "@/components/ui/button";

// mirrors the ProgressPayload struct defined in lib.rs
// the ? is like Option<>
type ProgressPayload = {
  event: string;
  percent?: number;
  message?: string;
};

// a union type for the download lifecycle
type DownloadStatus = "idle" | "downloading" | "converting" | "done" | "error" | "cancelled";

function App() {
  //these are current value / setter pairs
  const [url, setUrl] = useState("");
  const [outputDir, setOutputDir] = useState("");

  // download lifecycle starts as idle
  const [status, setStatus] = useState<DownloadStatus>("idle");

  // how far along the download is, 0 to 100
  const [percent, setPercent] = useState(0);

  // may hold an error description, null otherwise
  const [message, setMessage] = useState<string | null>(null);

  // register the Tauri event listener once
  useEffect(() => {
    // listen() to the "download://progress" channel
    // every time Rust calls app.emit("download://progress", ...) the callback here fires
    // it returns a Promise that resolves to an unsubscribe function
    const unlistenPromise = listen<ProgressPayload>("download://progress", (event) => {
      const p = event.payload;

      if (p.event === "downloading" && p.percent !== undefined) {
        setStatus("downloading");
        setPercent(p.percent);
      } else if (p.event === "converting") {
        setStatus("converting");
      } else if (p.event === "done") {
        setStatus("done");
        setPercent(100);
      } else if (p.event === "error") {
        setStatus("error");
        setMessage(p.message ?? "an unknown error occurred");
      } else if (p.event === "cancelled") {
        setStatus("cancelled");
      }
    });

    // so it doesn't leak after the window closes
    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  // async/await here to pause until the promise resolves
  async function startDownload() {
    // reset to a clean slate before each new download
    setStatus("downloading");
    setPercent(0);
    setMessage(null);

    try {
      // call fn from download.rs
      await invoke("start_download", { url, outputDir });
    } catch (e) {
      setStatus("error");
      setMessage(String(e));
    }
  }

  async function cancelDownload() {
    try {
      await invoke("cancel_download");
    } catch {
      // if there was nothing running, cancel is a no-op from the user's perspective
    }
  }

  // true while yt-dlp is running or ffmpeg is converting
  // i derive this from status rather than storing another boolean
  // because single source of truth => always in sync
  const isActive = status === "downloading" || status === "converting";

  return (
    <main className="min-h-screen bg-background flex items-center justify-center p-8">
      <div className="w-full max-w-md flex flex-col gap-6">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">MpGrab</h1>
          <p className="text-sm text-muted-foreground mt-1">Download YouTube videos as MP3</p>
        </div>

        <div className="flex flex-col gap-4">
          <div className="flex flex-col gap-1.5">
            {/* htmlFor links this label to the input. clicking the label focuses the field */}
            <label htmlFor="url-input" className="text-sm font-medium">
              YouTube URL
            </label>
            <input
              id="url-input"
              type="text"
              placeholder="https://www.youtube.com/watch?v=..."
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              disabled={isActive}
              className="h-9 rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-xs transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
            />
          </div>

          <div className="flex flex-col gap-1.5">
            <label htmlFor="dir-input" className="text-sm font-medium">
              Output folder
            </label>
            <input
              id="dir-input"
              type="text"
              placeholder="/home/petros/Music"
              value={outputDir}
              onChange={(e) => setOutputDir(e.target.value)}
              disabled={isActive}
              className="h-9 rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-xs transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
            />
          </div>
        </div>

        <div className="flex gap-3">
          <Button
            onClick={startDownload}
            // prevents clicks when a download is running or the fields are empty
            disabled={isActive || url.trim() === "" || outputDir.trim() === ""}
            className="flex-1"
          >
            {isActive ? (status === "converting" ? "Converting..." : "Downloading...") : "Download"}
          </Button>

          {/* cancel button only appears while a download is active.
             this evaluates to the element when true, nothing when false */}
          {isActive && (
            <Button variant="destructive" onClick={cancelDownload}>
              Cancel
            </Button>
          )}
        </div>

        {/* progress bar, only shown during the downloading phase */}
        {status === "downloading" && (
          <div className="flex flex-col gap-1.5">
            <div className="flex justify-between text-xs text-muted-foreground">
              <span>Downloading</span>
              <span>{percent.toFixed(1)}%</span>
            </div>
            <div className="h-2 w-full rounded-full bg-muted overflow-hidden">
              <div
                className="h-full bg-primary transition-all duration-300"
                style={{ width: `${percent}%` }}
              />
            </div>
          </div>
        )}

        {/* one status message per outcome, shown below the progress bar */}
        {status === "converting" && (
          <p className="text-sm text-muted-foreground">Converting to MP3...</p>
        )}
        {status === "done" && (
          <p className="text-sm text-green-600 dark:text-green-400">
            Done! File saved to {outputDir}
          </p>
        )}
        {status === "error" && <p className="text-sm text-destructive">{message}</p>}
        {status === "cancelled" && (
          <p className="text-sm text-muted-foreground">Download cancelled.</p>
        )}
      </div>
    </main>
  );
}

// export default makes this component importable by main.tsx
export default App;
