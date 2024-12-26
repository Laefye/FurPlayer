import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"

export type Metadata = {
    id: number,
    title: string,
    author: string,
    platform: {
        YouTube?: string,
    }
}

export type File = {
    bytes: number[],
    mime: string,
}

export type Audio = {
    metadata: Metadata,
    data: {
        Urled?: {
            thumbnail: string,
            audio: string,
        },
        Loaded?: {
            audio: File,
            thumbnail: File,
        },
    }
}

export async function getPlaylistMetadata(): Promise<Metadata[]> {
    return await invoke("get_playlist_metadata");
}

export async function addNewAudio(url: string): Promise<Audio> {
    return await invoke("add_new_audio", { url });
}

export async function removeAudio(id: number) {
    return await invoke("remove_audio", { id });
}

export async function loadAudio(id: number): Promise<Audio> {
    return await invoke("load_audio", { id });
}
