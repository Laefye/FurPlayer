import { createRef, useEffect, useRef, useState } from "react";
import reactLogo from "./assets/react.svg";
import { addNewAudio, Audio, getPlaylistMetadata, loadAudio, Metadata, removeAudio } from "./Engine";
import { Playlist } from "./Playlist";
import { listen } from "@tauri-apps/api/event";


function App() {
  let [playlist, setPlaylist] = useState<Metadata[]>([]);
  let [url, setUrl] = useState("");
  let [audioData, setAudioData] = useState<Audio | null>(null);
  let [loading, setLoading] = useState(false);
  let [thumbnail, setThumbnail] = useState<string | null>(null)
  let [audio, setAudio] = useState<string | null>(null);
  let [downloadingTable, setDownloadingTable] = useState<{[id: number]: number}>({});
  let audioRef = useRef<HTMLAudioElement | null>(null);

  useEffect(() => {
    let unlisten = listen('status_download', (e) => {
        let payload: any = e.payload;
        let table = {...downloadingTable}
        if (payload.Started) {
          table[payload.Started] = 0;
        }
        if (payload.Process) {
          table[payload.Process[0]] = payload.Process[1]
        }
        if (payload.Finished) {
          delete table[payload.Finished];
        }
        setDownloadingTable(table);
        return () => {
          (async () => {
            (await unlisten)();
          })();
        };
    });
  }, []);

  useEffect(() => {
    if (audioData) {
      if (audioData.data.Urled) {
        setThumbnail(audioData.data.Urled.thumbnail);
        setAudio(audioData.data.Urled.audio);
      } else if (audioData.data.Loaded) {
        setThumbnail(URL.createObjectURL(new Blob([new Uint8Array(audioData.data.Loaded.thumbnail.bytes)], { type: audioData.data.Loaded.thumbnail.mime })));
        setAudio(URL.createObjectURL(new Blob([new Uint8Array(audioData.data.Loaded.audio.bytes)], { type: audioData.data.Loaded.audio.mime })));
      }
    }
  }, [audioData]);

  useEffect(() => {
    (async () => {
      setPlaylist(await getPlaylistMetadata());
    })();
  }, []);

  async function _addPlaylist(url: string) {
    setLoading(true);
    let audio = await addNewAudio(url);
    setLoading(false);
    setPlaylist([...playlist, audio.metadata]);
    setAudioData(audio);
  }

  async function _loadAudio(id: number) {
    setLoading(true);
    let audio = await loadAudio(id);
    setLoading(false);
    setAudioData(audio);
  }

  async function _removeAudio(id: number) {
    await removeAudio(id);
    if (audioData.metadata.id == id) {
      setAudioData(null);
    }
    setPlaylist(playlist.filter(metadata => metadata.id !== id));
  }

  return (
    <main className="bg-gray-900 text-white h-screen flex flex-col">
      <div className="flex justify-center p-4">
      <form className="flex flex-col w-full" onSubmit={e => {e.preventDefault(); _addPlaylist(url);}}>
        <input onChange={(e) => setUrl(e.target.value)} type="url" placeholder="YouTube url" className="p-2 px-3 rounded-lg bg-gray-800 text-white outline-none" />
      </form>
      </div>
      <Playlist playlist={playlist} onSelect={_loadAudio} onRemove={_removeAudio} selectedId={audioData ? audioData.metadata.id : null} />
      <div className="p-4 bg-gray-800">
      { loading ? (
        <div className="flex items-center">
        <div className="w-28 h-28 me-3 bg-gray-700 animate-pulse rounded"></div>
        <div className="flex-grow bg-gray-700 animate-pulse h-12 rounded-lg"></div>
        </div>
      ) : (
        audioData && <div className="flex items-center">
          <div className="w-28 h-28 bg-center bg-cover flex-shrink-0 rounded me-3" style={{backgroundImage: `url(${thumbnail})`}}></div>
          <div className="flex-grow flex flex-col">
            <audio autoPlay src={audio} controls className="w-full" ref={audioRef}></audio>
            { downloadingTable[audioData.metadata.id] ? (<>
              <div className="text-slate-500 mt-2">Downloading:</div>
              <div className="bg-slate-700 h-2 rounded-lg w-full">
                <div className="bg-slate-500 h-2 rounded-lg" style={{width: `${downloadingTable[audioData.metadata.id] * 100}%`}}></div>
              </div>
            </>) : (<></>)}
          </div>
        </div>
      )}
      </div>
    </main>
  );
}

export default App;
