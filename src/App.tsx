import { useEffect, useState } from "react";
// import { addNewAudio, Audio, getPlaylistMetadata, loadAudio, AudioDTO, removeAudio } from "./Engine";
import { Playlist } from "./Playlist";
import { IndexedAudioDTO, useEngine } from "./Engine";
import { Player } from "./components/Player";
import { DownloadList } from "./components/DownloadList";


function App() {
  let [url, setUrl] = useState<string>('');
  let engine = useEngine();

  return (
    <main className="bg-gray-900 h-screen text-white p-2 app flex flex-col">
      <form onSubmit={(e) => {e.preventDefault(); engine.addAudio(url);}} className="flex search">
        <input type="url" onChange={(e) => setUrl(e.target.value)} className="outline-none w-full bg-gray-800 p-2 px-3 rounded-xl" placeholder="Enter a youtube URL"/>
      </form>
      <Playlist/>
      <DownloadList/>
      <Player/>
    </main>
  );
}

export default App;
