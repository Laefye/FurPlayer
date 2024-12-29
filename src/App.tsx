import { useEffect, useState } from "react";
// import { addNewAudio, Audio, getPlaylistMetadata, loadAudio, AudioDTO, removeAudio } from "./Engine";
import { Playlist } from "./Playlist";
import { IndexedAudioDTO, useEngine } from "./Engine";
import { Player } from "./components/Player";
import { DownloadList } from "./components/DownloadList";
import AddAudio from "./components/AddAudio";


function App() {
  let [url, setUrl] = useState<string>('');
  let engine = useEngine();

  return (
    <main className="bg-gray-900 h-screen text-white p-3 app flex flex-col">
      <AddAudio/>
      <Playlist/>
      <DownloadList/>
      <Player/>
    </main>
  );
}

export default App;
