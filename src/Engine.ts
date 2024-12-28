import { invoke } from "@tauri-apps/api/core"

export type ContentDTO = {
    Url?: string,
    Local?: {
        bytes: number[],
        mime: string,
    }
}

export type AudioDTO = {
    id: number,
    title: string,
    author: string,
    source: {
        YouTube?: string,
    },
    thumbnail: ContentDTO,
    media: ContentDTO,
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
    return await invoke("get_playlist_metadata");
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

export async function loadAudio(id: number): Promise<AudioDTO | null> {
    return await invoke("load_audio", { id });
}

export async function getMedia(id: number): Promise<ContentDTO> {
    return await invoke("get_media", { id });
}

export async function getThumbnail(id: number): Promise<ContentDTO> {
    return await invoke("get_thumbnail", { id });
}
