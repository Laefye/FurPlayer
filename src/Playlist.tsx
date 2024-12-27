import { AudioDTO, IndexedAudioDTO } from "./Engine";

export function Playlist({ playlist, onSelect, onRemove, selectedId }: { playlist: IndexedAudioDTO[], onSelect: (id: number) => void, onRemove: (id: number) => void, selectedId: {id: number, loading: boolean} | null }) {
    function getSource(metadata: IndexedAudioDTO) {
        if (metadata.source.YouTube) {
            return "YouTube";
        }
        return "";
    }
    function getSourceLink(metadata: IndexedAudioDTO) {
        if (metadata.source.YouTube) {
            return metadata.source.YouTube;
        }
        return "";
    }
    return (
        <div className="flex-grow flex overflow-auto">
            <div className="flex flex-col w-full">
            { playlist.map((audio, index) => (
            <div key={index} className="relative border-b border-gray-700">
                <div className={"absolute top-0 left-0 w-full h-full " + (selectedId && selectedId.id === audio.id ? (selectedId.loading ? "bg-gray-800 animate-pulse" : "bg-gray-800") : "")}>

                </div>
                <div className="flex relative z-10">
                    <a className="px-3 flex ms-3 self-center items-center justify-center bg-red-500 p-1 rounded" href={getSourceLink(audio)} target="_blank">{getSource(audio)}</a>
                    <button onClick={() => onSelect(audio.id)} className="p-2 px-3 bg-transparent flex-grow flex flex-col items-center">
                        <span className="text-white">{audio.title}</span>
                        <span className="text-gray-400 text-sm">{audio.author}</span>
                    </button>
                    <button className="text-red-400 p-2 px-3" onClick={() => onRemove(audio.id)}>
                        Delete
                    </button>
                </div>
            </div>
            )) }
            </div>
        </div>
    )
}