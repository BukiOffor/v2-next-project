import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

type UpdateMetadata = {
  version: string;
  currentVersion: string;
};

type DownloadEvent =
  | { event: 'started'; data: { contentLength: number | null } }
  | { event: 'progress'; data: { chunkLength: number } }
  | { event: 'finished' };

export const UpdateChecker = () => {
  const [updateAvailable, setUpdateAvailable] = useState<UpdateMetadata | null>(null);
  const [downloading, setDownloading] = useState(false);
  const [progress, setProgress] = useState<number>(0);
  const [error, setError] = useState<string | null>(null);

  const checkForUpdate = async () => {
    try {
      const metadata = await invoke<UpdateMetadata | null>('fetch_update');
      setUpdateAvailable(metadata);
    } catch (err) {
      console.error('Failed to check for updates:', err);
      setError('Unable to check for updates.');
    }
  };

  const installUpdate = async () => {
    setDownloading(true);
    let total = 0;

    const unlisten = await listen<DownloadEvent>('install_update', (event) => {
      switch (event.payload.event) {
        case 'started':
          console.log('Download started', event.payload.data);
          break;
        case 'progress':
          total += event.payload.data.chunkLength;
          setProgress(total);
          break;
        case 'finished':
          console.log('Download finished');
          setDownloading(false);
          break;
      }
    });

    try {
      await invoke('install_update', {
        on_event: (event: DownloadEvent) => {
            switch (event.event) {
            case 'started':
                console.log('Started', event.data);
                break;
            case 'progress':
                console.log('Progress', event.data.chunkLength);
                break;
            case 'finished':
                console.log('Finished');
                break;
            }
        }
});
;
    } catch (err) {
      console.error('Install error:', err);
      setError('Update installation failed.');
    } finally {
      unlisten();
    }
  };

  useEffect(() => {
    checkForUpdate();
  }, []);

  return (
    <div className="p-4 max-w-md mx-auto">
      <h2 className="text-xl font-bold mb-2">Software Update</h2>
      {error && <p className="text-red-500">{error}</p>}
      {updateAvailable ? (
        <div>
          <p>
            New version available: <strong>{updateAvailable.version}</strong><br />
            Current version: {updateAvailable.currentVersion}
          </p>
          <button
            onClick={installUpdate}
            disabled={downloading}
            className="mt-4 px-4 py-2 bg-blue-600 text-white rounded"
          >
            {downloading ? 'Installing...' : 'Install Update'}
          </button>
          {downloading && <p className="mt-2">Downloaded: {progress} bytes</p>}
        </div>
      ) : (
        <p>No updates available.</p>
      )}
    </div>
  );
};
