import { invoke } from "@tauri-apps/api/core"
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


export async function getPlaylistMetadata(): Promise<IndexedAudioDTO[]> {
    return await invoke("get_playlist");
}

export async function addNewAudio(url: string): Promise<IndexedAudioDTO> {
    try {
        return await invoke("add_new_audio", { url });
    } catch (e) {
        console.log(e);
    }
}

export async function removeAudio(id: number) {
    return await invoke("remove_audio", { id });
}

export async function getMedia(id: number): Promise<ContentDTO> {
    return await invoke("get_media", { id });
}

export async function getThumbnail(id: number): Promise<ContentDTO> {
    return await invoke("get_thumbnail", { id });
}

export type ThumbnailEvent = {
    id: number,
    url: string,
    currentThumbnails: {[id: number]: string},
}

export default class Engine {
    private listeners: {[type: string]: ((e: any)=>void)[]};
    private playlist: IndexedAudioDTO[];
    private thumbnails: {[id: number]: ContentDTO};

    constructor() {
        this.listeners = {
            'thumbnail_load': [],
        };
        this.playlist = [];
        this.thumbnails = {};
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
        this.playlist = await invoke("get_playlist_metadata");
        this.thumbnails = {};
        for (const audio of this.playlist) {
            this.loadThumbnail(audio.id);
        }
        return this.playlist;
    }

    async addAudio(url: string): Promise<IndexedAudioDTO> {
        let audio: IndexedAudioDTO = await invoke("add_new_audio", { url });
        this.loadThumbnail(audio.id);
        this.playlist.push(audio);
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
        this.playlist = this.playlist.filter((audio) => audio.id !== id);
    }

    get just_playlist(): IndexedAudioDTO[] {
        return [...this.playlist];
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

export function EngineContext({children}: {children: ReactNode}) {
    let [engine, setEngine] = useState<Engine>(new Engine());
    let [playlist, setPlaylist] = useState<IndexedAudioDTO[]>([]);
    let [thumbnails, setThumbnails] = useState<{[id: number]: string}>({});
    let [selectedAudio, setSelectedAudio] = useState<[IndexedAudioDTO, string] | null>(null);
    let [state, setState] = useState<State>('idle');
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
            await engine.addAudio(url);
            setPlaylist(engine.just_playlist);
            setState('idle');
        },
        removeAudio: async (id: number) => {
            await engine.removeAudio(id);
            setPlaylist(engine.just_playlist);
        },
        state,
        selectedAudio,
        selectAudio: async (id: number) => {
            setState('loading_audio');
            let media = await engine.getMedia(id);
            let audio = engine.just_playlist.find(audio => audio.id === id);
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
