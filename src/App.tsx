import { createRef, useEffect, useRef, useState } from "react";
import reactLogo from "./assets/react.svg";
// import { addNewAudio, Audio, getPlaylistMetadata, loadAudio, AudioDTO, removeAudio } from "./Engine";
import { Playlist } from "./Playlist";
import { listen } from "@tauri-apps/api/event";
import { addNewAudio, AudioDTO, getMedia, getPlaylistMetadata, getThumbnail, IndexedAudioDTO, loadAudio, removeAudio } from "./Engine";


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
      if (audioData.thumbnail.Url) {
        setThumbnail(audioData.thumbnail.Url);
      } else if (audioData.thumbnail.Local) {
        setThumbnail(
          URL.createObjectURL(new Blob([new Uint8Array(audioData.thumbnail.Local.bytes)], { type: audioData.thumbnail.Local.mime }))
        );
      }
      if (audioData.media.Url) {
        setAudio(audioData.media.Url);
      } else if (audioData.media.Local) {
        setAudio(
          URL.createObjectURL(new Blob([new Uint8Array(audioData.media.Local.bytes)], { type: audioData.media.Local.mime }))
        );
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
    setAudioData({
      id: audio.id,
      author: audio.author,
      media: await getMedia(audio.id),
      thumbnail: await getThumbnail(audio.id),
      source: audio.source,
      title: audio.title,
    });
    setSelectedId(audio.id);
  }

  async function _loadAudio(id: number) {
    setLoading(true);
    setSelectedId(id);
    let indexedAudio = playlist.find(x => x.id == id);
    let audio: AudioDTO = {
      id: indexedAudio.id,
      author: indexedAudio.author,
      media: await getMedia(indexedAudio.id),
      thumbnail: await getThumbnail(indexedAudio.id),
      source: indexedAudio.source,
      title: indexedAudio.title,
    }
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
