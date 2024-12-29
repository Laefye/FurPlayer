import { invoke } from "@tauri-apps/api/core"
import { listen, UnlistenFn } from "@tauri-apps/api/event"
import { createContext, ReactNode, useContext, useEffect, useState } from "react"

export type ContentDTO = {
    Url?: string,
    Local?: {
        bytes: number[],
        mime: string,
    }
}

export type IndexedAudioDTO = {
    id: number,
    title: string,
    author: string,
    source: {
        YouTube?: string,
    },
}

export type ThumbnailEvent = {
    id: number,
    url: string,
    currentThumbnails: {[id: number]: string},
}

export type ProcessDownloadEvent = {
    audio: IndexedAudioDTO,
}

export type PartDownloadEvent = ProcessDownloadEvent & {
    total: number,
    downloaded: number,
}


type DownloadEventDTO = {
    StartDownload?: {
        audio: IndexedAudioDTO,
    },
    FinishedDownload?: {
        audio: IndexedAudioDTO,
    },
    ErrorDownload?: {
        audio: IndexedAudioDTO,
        error: string,
    },
    Download?: {
        audio: IndexedAudioDTO,
        downloaded: number,
        total: number,
    }
}

export type Download = {
    state: 'downloading' | 'finished' | 'error',
    error: string | null,
    progress: {
        total: number,
        downloaded: number,
    } | undefined;
    audio: IndexedAudioDTO,
}

export default class Engine {
    private listeners: {[type: string]: ((e: any)=>void)[]};
    private _playlist: IndexedAudioDTO[];
    private thumbnails: {[id: number]: ContentDTO};
    private _downloads: {[id: number]: Download};

    private _drop: Promise<UnlistenFn>;

    constructor() {
        this.listeners = {
            'thumbnail_load': [],
            'download': [],
        };
        this._playlist = [];
        this.thumbnails = {};
        this._downloads = {};

        
    }

    init() {
        this._drop = listen('download', (e) => {
            let payload: DownloadEventDTO = e.payload;
            if (payload.StartDownload) {
                this._downloads[payload.StartDownload.audio.id] = {
                    state: 'downloading',
                    error: null,
                    progress: undefined,
                    audio: payload.StartDownload.audio,
                };
                for (const listener of this.listeners['download']) {
                    listener(payload.StartDownload);
                }
            } else if (payload.FinishedDownload) {
                this._downloads[payload.FinishedDownload.audio.id] = {
                    state: 'finished',
                    error: null,
                    progress: undefined,
                    audio: payload.FinishedDownload.audio,
                }
                for (const listener of this.listeners['download']) {
                    listener(payload.FinishedDownload);
                }
            } else if (payload.ErrorDownload) {
                this._downloads[payload.ErrorDownload.audio.id] = {
                    state: 'error',
                    error: payload.ErrorDownload.error,
                    progress: undefined,
                    audio: payload.ErrorDownload.audio,
                };
                for (const listener of this.listeners['download']) {
                    listener(payload.ErrorDownload);
                }
            } else if (payload.Download) {
                this._downloads[payload.Download.audio.id] = {
                    state: 'downloading',
                    error: null,
                    progress: {
                        downloaded: payload.Download.downloaded,
                        total: payload.Download.total,
                    },
                    audio: payload.Download.audio,
                }
                for (const listener of this.listeners['download']) {
                    listener(payload.Download);
                }
            }
        });
    }

    dispose() {
        (async() => {
            (await this._drop)();
        });
    }

    private contentToURL(content: ContentDTO): string {
        if (content.Url) {
            return content.Url;
        } else if (content.Local) {
            return URL.createObjectURL(new Blob([new Uint8Array(content.Local.bytes)], {type: content.Local.mime}))
        }
    }

    private async loadThumbnail(id: number) {
        let content: ContentDTO = await invoke("get_thumbnail", { id });
        this.thumbnails[id] = content;
        for (const thumbnail of this.listeners['thumbnail_load']) {
            let url = this.contentToURL(content);
            let thumbnails: {[id: number]: string} = {};
            for (const id in this.thumbnails) {
                thumbnails[id] = this.contentToURL(this.thumbnails[id]);
            }
            let event: ThumbnailEvent = {
                id,
                url,
                currentThumbnails: thumbnails,
            };
            thumbnail(event);
        }
    }

    async getMedia(id: number): Promise<string> {
        let media: ContentDTO = await invoke("get_media", { id });
        if (media.Url) {
            return media.Url;
        } else if (media.Local) {
            return URL.createObjectURL(new Blob([new Uint8Array(media.Local.bytes)], {type: media.Local.mime}))
        }
    }

    async getPlaylist(): Promise<IndexedAudioDTO[]> {
        this._playlist = await invoke("get_playlist");
        this.thumbnails = {};
        for (const audio of this._playlist) {
            this.loadThumbnail(audio.id);
        }
        return this._playlist;
    }

    async addAudio(url: string): Promise<IndexedAudioDTO> {
        let audio: IndexedAudioDTO = await invoke("add_new_audio", { url });
        this.loadThumbnail(audio.id);
        this._playlist.push(audio);
        return audio;
    }

    on(event: string, callback: (e: any) => void) {
        if (this.listeners[event]) {
            this.listeners[event].push(callback);
            return () => {
                const index = this.listeners[event].findIndex((cb) => cb === callback);
                if (index >= 0) {
                    this.listeners[event].splice(index, 1);
                }
            }
        }
    }

    async removeAudio(id: number) {
        await invoke("remove_audio", { id });
        this._playlist = this._playlist.filter((audio) => audio.id !== id);
    }

    get playlist(): IndexedAudioDTO[] {
        return [...this._playlist];
    }

    get downloads(): {[id: number]: Download} {
        return {...this._downloads};
    }
}

type State = 'idle' | 'fetching_audio' | 'loading_audio';

type ContextType = { 
    engine: Engine,
    playlist: IndexedAudioDTO[],
    thumbnails: {[id: number]: string},
    addAudio: (url: string) => void,
    removeAudio: (id: number) => void,
    selectAudio: (id: number) => void,
    state: 'idle' | 'fetching_audio' | 'loading_audio',
    selectedAudio: [IndexedAudioDTO, string] | null,
}

const Context = createContext<ContextType | undefined>(undefined);

export class FetchAudioError extends Error {
    constructor(message: string) {
        super(message);
        this.name = "FetchAudioError";
    }
}

export class NotFoundError extends Error {
    constructor(message: string) {
        super(message);
        this.name = "NotFoundError";
    }
}

export function EngineContext({children}: {children: ReactNode}) {
    let [engine, setEngine] = useState<Engine>(new Engine());
    let [playlist, setPlaylist] = useState<IndexedAudioDTO[]>([]);
    let [thumbnails, setThumbnails] = useState<{[id: number]: string}>({});
    let [selectedAudio, setSelectedAudio] = useState<[IndexedAudioDTO, string] | null>(null);
    let [state, setState] = useState<State>('idle');
    let [downloads, setDownloads] = useState<{[id: number]: Download}>({});
    
    useEffect(() => {
        engine.init();
        return () => engine.dispose();
    }, []);

    useEffect(() => {
        return engine.on('download', (e) => {
            setDownloads(engine.downloads);
        })
    }, []);
    
    useEffect(() => {
        engine.getPlaylist().then(setPlaylist);
    }, []);
    useEffect(() => {
        return engine.on('thumbnail_load', (e: ThumbnailEvent) => {
            setThumbnails(e.currentThumbnails);
        });
    }, []);
    let contextType: ContextType = {
        engine,
        playlist,
        thumbnails,
        addAudio: async (url: string) => {
            setState('fetching_audio');
            try {
                await engine.addAudio(url);
                setPlaylist(engine.playlist);
                setState('idle');
            } catch (error) {
                setState('idle');
                if (error === 'Bad link') {
                    throw new FetchAudioError("The link provided is not a valid link");
                } else if (error === 'Not found') {
                    throw new NotFoundError("The link provided does not point to a valid audio");
                } else {
                    throw error;
                }
            }
        },
        removeAudio: async (id: number) => {
            await engine.removeAudio(id);
            setPlaylist(engine.playlist);
        },
        state,
        selectedAudio,
        selectAudio: async (id: number) => {
            setState('loading_audio');
            let media = await engine.getMedia(id);
            let audio = engine.playlist.find(audio => audio.id === id);
            if (audio) {
                setSelectedAudio([audio, media]);
            }
            setState('idle');
        }
    }
    return <Context.Provider value={contextType}>
        {children}
    </Context.Provider>
}

export function useEngine() {
    return useContext(Context);
}
