import { invoke } from "@tauri-apps/api/core"

export type Metadata = {
    id: number,
    title: string,
    author: string,
}

export type Audio = {
    metadata: Metadata,
    data: {
        Urled?: {
            thumbnail: string,
            audio: string,
        }
    }
}

export async function getPlaylistMetadata(): Promise<Metadata[]> {
    return await invoke("get_playlist_metadata");
}

export async function addPlaylist(url: string): Promise<Audio> {
    return await invoke("add_playlist", { url });
}

export async function loadAudio(id: number): Promise<Audio> {
    return await invoke("load_audio", { id });
}
