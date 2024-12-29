import { useEngine } from "../Engine";

export function DownloadList() {
    const { playlist, engine } = useEngine();
    let downloads = engine.downloads;
    
    return (
        <div className="bg-gray-800 p-4 rounded-xl downloads">
            <h2 className="text-lg font-bold mb-4">Downloads</h2>
            <ul className="space-y-2">
                {Object.entries(downloads).map(([id, download]) => {
                    const audio = playlist.find((audio) => audio.id === parseInt(id));
                    return (
                        <li key={id} className="flex flex-col space-y-1">
                            <span className="font-semibold">{audio?.title}</span>
                            <span className="text-sm text-gray-400">{audio?.author}</span>
                            {download.state === "downloading" && download.progress && (
                                <div className="w-full bg-gray-700 rounded-full h-2.5">
                                    <div
                                        className="bg-blue-500 h-2.5 rounded-full"
                                        style={{
                                            width: `${(download.progress.downloaded / download.progress.total) * 100}%`,
                                        }}
                                    ></div>
                                </div>
                            )}
                            {download.state === "finished" && (
                                <span className="text-green-500 text-sm">Download finished</span>
                            )}
                            {download.state === "error" && (
                                <span className="text-red-500 text-sm">Error: {download.error}</span>
                            )}
                        </li>
                    );
                })}
            </ul>
        </div>
    );
}