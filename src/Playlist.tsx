import { AudioDTO, IndexedAudioDTO } from "./Engine";

export function Playlist({ playlist, onSelect, onRemove, selectedId }: { playlist: IndexedAudioDTO[], onSelect: (id: number) => void, onRemove: (id: number) => void, selectedId: number | null }) {
    return (
        <div className="flex-grow flex overflow-auto">
            <div className="flex flex-col w-full">
            { playlist.map((metadata, index) => (
            <div key={index} className={"flex border-b border-gray-700 " + (selectedId === metadata.id ? "bg-slate-800" : "")}>
                <span className="px-3 h-full flex items-center justify-center">{metadata.source}</span>
                <button onClick={() => onSelect(metadata.id)} className="p-2 px-3 bg-transparent flex-grow flex flex-col items-center">
                    <span className="text-white">{metadata.title}</span>
                    <span className="text-gray-400 text-sm">{metadata.author}</span>
                </button>
                <button className="text-red-400 p-2 px-3" onClick={() => onRemove(metadata.id)}>
                    Delete
                </button>
            </div>
            )) }
            </div>
        </div>
    )
}