import { createRef, useEffect, useRef, useState } from "react";
import reactLogo from "./assets/react.svg";
// import { addNewAudio, Audio, getPlaylistMetadata, loadAudio, AudioDTO, removeAudio } from "./Engine";
import { Playlist } from "./Playlist";
import { listen } from "@tauri-apps/api/event";
import { addNewAudio, AudioDTO, getPlaylistMetadata, IndexedAudioDTO, loadAudio, removeAudio } from "./Engine";


function App() {
  let [playlist, setPlaylist] = useState<IndexedAudioDTO[]>([]);
  let [url, setUrl] = useState("");
  let [audioData, setAudioData] = useState<AudioDTO | null>(null);
  let [loading, setLoading] = useState(false);
  let [selectedId, setSelectedId] = useState<number | null>(null);
  let [thumbnail, setThumbnail] = useState<string | null>(null)
  let [audio, setAudio] = useState<string | null>(null);
  let audioRef = useRef<HTMLAudioElement | null>(null);

  useEffect(() => {
    if (audioData) {
      if (audioData.content.Url) {
        setThumbnail(audioData.content.Url.thumbnail);
        setAudio(audioData.content.Url.media);
      } else if (audioData.content.Local) {
        setThumbnail(URL.createObjectURL(new Blob([new Uint8Array(audioData.content.Local.thumbnail.bytes)], { type: audioData.content.Local.thumbnail.mime })));
        setAudio(URL.createObjectURL(new Blob([new Uint8Array(audioData.content.Local.media.bytes)], { type: audioData.content.Local.media.mime })));
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
    setSelectedId(null);
    let audio = await addNewAudio(url);
    setLoading(false);
    setPlaylist([...playlist, {id: audio.id, title: audio.title, author: audio.author, source: audio.source}]);
    setAudioData(audio);
    setSelectedId(audio.id);
  }

  async function _loadAudio(id: number) {
    setLoading(true);
    setSelectedId(id);
    let audio = await loadAudio(id);
    setLoading(false);
    setAudioData(audio);
  }

  async function _removeAudio(id: number) {
    await removeAudio(id);
    if (audioData.id == id) {
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
      <Playlist playlist={playlist} onSelect={_loadAudio} onRemove={_removeAudio} selectedId={selectedId ? {id: selectedId, loading} : null} />
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
          </div>
        </div>
      )}
      </div>
    </main>
  );
}

export default App;
