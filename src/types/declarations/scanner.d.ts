type ScannedSong = { song: Song }
type ScannedPlaylist = { filePath: string; title: string; songs: Song[] }

type ScanWorkerWorkerType = {
  scanSinglePlaylist: (path: string, splitPattern: string, loggerPath: string) => ScannedPlaylist

  scanSingleSong: (path: string, splitPattern: string, loggerPath: string) => ScannedSong
  getHash: (path: string, loggerPath: string) => string
  getCover: (
    path: string,
    basePath: string,
    id: string,
    onlyHigh: boolean,
    loggerPath: string,
  ) => { high: string; low?: string } | undefined
}

type ScanWorker = Awaited<ReturnType<typeof spawn<ScanWorkerWorkerType>>>
