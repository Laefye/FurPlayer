import { createRef, useEffect, useState } from "react";
import reactLogo from "./assets/react.svg";
import { addPlaylist, Audio, getPlaylistMetadata, loadAudio, Metadata } from "./Engine";


function App() {
  let [playlist, setPlaylist] = useState<Metadata[]>([]);
  let [url, setUrl] = useState("");
  let [audioData, setAudioData] = useState<Audio | null>(null);
  let [loading, setLoading] = useState(false);
  
  let thumbnail;
  let audio;
  if (audioData) {
    if (audioData.data.Urled) {
      thumbnail = audioData.data.Urled.thumbnail;
      audio = audioData.data.Urled.audio;
    } else if (audioData.data.Loaded) {
      thumbnail = URL.createObjectURL(new Blob([new Uint8Array(audioData.data.Loaded.thumbnail.bytes)], { type: audioData.data.Loaded.thumbnail.mime }));
      audio = URL.createObjectURL(new Blob([new Uint8Array(audioData.data.Loaded.audio.bytes)], { type: audioData.data.Loaded.audio.mime }));
    }
  }

  useEffect(() => {
    (async () => {
      setPlaylist(await getPlaylistMetadata());
    })();
  }, []);

  async function _addPlaylist(url: string) {
    setLoading(true);
    let audio = await addPlaylist(url);
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

  return (
    <main className="bg-gray-900 text-white h-screen flex flex-col">
      <div className="flex justify-center p-4">
      <form className="flex flex-col w-full" onSubmit={e => {e.preventDefault(); _addPlaylist(url);}}>
        <input onChange={(e) => setUrl(e.target.value)} type="url" placeholder="YouTube url" className="p-2 px-3 rounded-lg bg-gray-800 text-white outline-none" />
      </form>
      </div>
      <div className="flex-grow flex overflow-auto">
      <div className="flex flex-col w-3/5 mx-auto">
        { playlist.map((metadata, index) => (
        <button key={index} onClick={() => _loadAudio(metadata.id)} className="p-2 px-3 bg-transparent flex flex-col items-center border-b border-gray-700">
          <span className="text-white">{metadata.title}</span>
          <span className="text-gray-400 text-sm">{metadata.author}</span>
        </button>
        )) }
      </div>
      </div>
      <div className="p-4">
      { loading ? (
        <div className="flex flex-col items-center">
        <div className="w-1/4 mb-4 bg-gray-700 animate-pulse h-48 rounded-lg"></div>
        <div className="w-full bg-gray-700 animate-pulse h-12 rounded-lg"></div>
        </div>
      ) : (
        audioData && <div className="flex flex-col items-center">
        <img src={thumbnail} alt="thumbnail" className="w-1/4 mb-4" />
        <audio src={audio} controls className="w-full"></audio>
        </div>
      )}
      </div>
    </main>
  );
}

export default App;
