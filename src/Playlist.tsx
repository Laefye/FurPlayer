import { useEngine } from "./Engine";
import { Thumbnail } from "./components/Thumbnail";

export function Playlist() {
    let { thumbnails, playlist, state, removeAudio, selectAudio, selectedAudio } = useEngine();
    return <div className="flex flex-col min-h-0 playlist">
            <h2 className="text-lg mb-2">Playlist</h2>
            <ul className="flex flex-col space-y-2 bg-gray-800 p-2 rounded-xl overflow-y-auto min-h-0 flex-grow">
            {playlist.map((audio, index) => (<li key={index} className={"flex last:border-b-0 border-b border-gray-700 p-2 hover:bg-gray-700 rounded transition-all " + ((selectedAudio && selectedAudio[0].id == audio.id) && "bg-gray-700")}>
                <button className="flex items-center space-x-2 flex-grow text-left" onClick={() => selectAudio(audio.id)}>
                    { (thumbnails == null || !(audio.id in thumbnails)) && (<div className="w-14 h-14 bg-gray-600 rounded animate-pulse"></div>)}
                    { (thumbnails != null && audio.id in thumbnails) && (<Thumbnail className="w-14 h-14" src={thumbnails[audio.id]}/>)}
                    <div className="flex flex-col">
                        <span>{audio.title}</span>
                        <span>{audio.author}</span>
                    </div>
                </button>
                <button className="ml-4 px-2 py-2 self-center bg-blue-500 text-white rounded hover:bg-blue-600 transition" onClick={() => removeAudio(audio.id)}>
                    Remove
                </button>
            </li>))}
            { state == 'fetching_audio' && <div className="flex last:border-b-0 border-b border-gray-700 p-2">
                <div className="flex items-center space-x-2 flex-grow text-left">
                    <div className="w-14 h-14 bg-gray-600 rounded animate-pulse"></div>
                    <div className="flex flex-col space-y-1 items-center">
                        <span className="w-20 h-4 bg-gray-600 rounded animate-pulse"></span>
                        <span className="w-20 h-4 bg-gray-600 rounded animate-pulse"></span>
                    </div>
                </div>
            </div> }
        </ul>
    </div>;
}