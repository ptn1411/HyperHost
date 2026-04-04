import { useState, useEffect } from "react";
import { check, Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export function UpdateDialog() {
  const [updateInfo, setUpdateInfo] = useState<Update | null>(null);
  const [isUpdating, setIsUpdating] = useState(false);
  const [progress, setProgress] = useState<{ downloaded: number; total: number } | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // Check for updates shortly after mount
    const timer = setTimeout(() => {
      checkForUpdate();
    }, 2000);
    return () => clearTimeout(timer);
  }, []);

  const checkForUpdate = async () => {
    try {
      const update = await check();
      if (update && update.available) {
        setUpdateInfo(update);
      }
    } catch (e) {
      console.error("Failed to check for updates", e);
    }
  };

  const startUpdate = async () => {
    if (!updateInfo) return;
    setIsUpdating(true);
    setError(null);
    try {
      let downloadedBytes = 0;
      let totalBytes = 0;
      
      await updateInfo.downloadAndInstall((event) => {
        switch (event.event) {
          case "Started":
            totalBytes = event.data.contentLength || 0;
            setProgress({ downloaded: 0, total: totalBytes });
            break;
          case "Progress":
            downloadedBytes += event.data.chunkLength;
            setProgress({ downloaded: downloadedBytes, total: totalBytes });
            break;
          case "Finished":
            // Done
            break;
        }
      });
      // Relaunch app
      await relaunch();
    } catch (err: any) {
      console.error(err);
      setError(String(err));
      setIsUpdating(false);
    }
  };

  if (!updateInfo) return null;

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4 animate-in fade-in">
      <div className="bg-surface border border-surface-3/30 rounded-2xl shadow-xl max-w-md w-full overflow-hidden">
        <div className="p-6">
          <div className="w-12 h-12 bg-accent/20 text-accent rounded-full flex items-center justify-center mb-4">
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
            </svg>
          </div>
          <h2 className="text-xl font-bold text-text mb-2">Update Available!</h2>
          <p className="text-text-muted mb-4">
            HyperHost <span className="font-mono text-xs">{updateInfo.version}</span> is ready to install.
          </p>
          
          {updateInfo.body && (
            <div className="bg-surface-2 p-3 rounded-lg text-sm text-text-muted mb-6 max-h-32 overflow-y-auto font-mono">
              {updateInfo.body}
            </div>
          )}

          {error && (
            <div className="bg-red-500/10 text-red-400 p-3 rounded-lg mb-4 text-sm font-mono whitespace-pre-wrap">
              {error}
            </div>
          )}

          {isUpdating && progress ? (
            <div className="mb-6">
              <div className="flex justify-between text-xs text-text-muted mb-2 font-mono">
                <span>Downloading...</span>
                <span>
                  {progress.total > 0
                    ? Math.round((progress.downloaded / progress.total) * 100)
                    : 0}%
                </span>
              </div>
              <div className="h-2 bg-surface-3 rounded-full overflow-hidden">
                <div
                  className="h-full bg-accent transition-all duration-200"
                  style={{
                    width: progress.total > 0 ? `${(progress.downloaded / progress.total) * 100}%` : "0%",
                  }}
                />
              </div>
            </div>
          ) : null}

          <div className="flex gap-3 mt-6">
            <button
              onClick={() => setUpdateInfo(null)}
              disabled={isUpdating}
              className="px-4 py-2 text-text-muted hover:text-text hover:bg-surface-3/50 rounded-lg transition-colors disabled:opacity-50"
            >
              Later
            </button>
            <button
              onClick={startUpdate}
              disabled={isUpdating}
              className="flex-1 bg-accent/10 hover:bg-accent/20 text-accent border border-accent/20 rounded-lg px-4 py-2 font-medium transition-colors focus:ring-2 disabled:opacity-50 flex justify-center items-center gap-2"
            >
              {isUpdating ? (
                <>
                  <svg className="animate-spin w-4 h-4" viewBox="0 0 24 24" fill="none">
                    <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                    <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                  Updating...
                </>
              ) : (
                "Install & Relaunch"
              )}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
