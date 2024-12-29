import { useState } from "react";
import { FetchAudioError, NotFoundError, useEngine } from "../Engine";

export default function AddAudio() {
  let [url, setUrl] = useState<string>('');
  let engine = useEngine();
  let [error, setError] = useState<string | null>(null);

  async function addAudio() {
    setError(null);
    try {
        await engine.addAudio(url);
    } catch (e) {
        if (e instanceof FetchAudioError || e.name === 'FetchAudioError') {
            setError(e.message);
        } else if (e instanceof NotFoundError || e.name === 'NotFoundError') {
            setError(e.message);
        } else {
            throw e;
        }
    }
  }

  return (
    <form onSubmit={(e) => {e.preventDefault(); addAudio();}} className="flex flex-col search">
        <input type="url" onChange={(e) => setUrl(e.target.value)} className="outline-none w-full bg-gray-800 p-2 px-3 rounded-xl" placeholder="Enter a youtube URL"/>
        {error && <p className="text-red-500">{error}</p>}
    </form>
  );
}