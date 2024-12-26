import { createRef, useEffect, useState } from "react";
import reactLogo from "./assets/react.svg";
import { addPlaylist, Audio, getPlaylistMetadata, loadAudio, Metadata } from "./Engine";


function App() {
  let [playlist, setPlaylist] = useState<Metadata[]>([]);
  let [url, setUrl] = useState("");
  let [audioData, setAudioData] = useState<Audio | null>(null);
  
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
    let audio = await addPlaylist(url);
    setPlaylist([...playlist, audio.metadata]);
  }

  async function _loadAudio(id: number) {
    let audio = await loadAudio(id);
    setAudioData(audio);
  }

  return (
    <main className="bg-gray-900 text-white h-screen flex flex-col">
      <div className="flex">
        <form className="flex flex-col w-3/5 mx-auto" onSubmit={e => {e.preventDefault(); _addPlaylist(url);}}>
          <input onChange={(e) => setUrl(e.target.value)} type="url" placeholder="YouTube url" className="p-2 px-3 rounded-lg bg-gray-800 text-white outline-none" />
        </form>
      </div>
      <div className="flex">
        <div className="flex flex-col w-1/5">
          { playlist.map((metadata, index) => (<button key={index} onClick={() => _loadAudio(metadata.id)} className="p-2 px-3 bg-transparent flex flex-col items-center"><span className="text-white">{metadata.title}</span><span className="text-gray-400 text-sm">{metadata.author}</span></button>)) }
        </div>
        <div className="flex-grow">
          { audioData && <div className="flex flex-col items-center">
            <img src={thumbnail} alt="thumbnail" className="w-1/2" />
            <audio src={audio} controls></audio>
          </div> }
        </div>
      </div>
    </main>
  );
}

export default App;
