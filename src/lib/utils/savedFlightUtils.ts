import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { libraryEvents } from './libraryEvents.svelte';

type EntryResponse = {
    status: string,
    errorMessage: string,
    data: string[],
}

export interface SavedFlight {
    path: string,
    name: string,
    extension: string,
    sizeBytes: number,
    modifiedMs: number | null,
}

type DetailsResponse = {
    ok: boolean,
    errorMessage: string,
    data: SavedFlight[],
}

export async function loadEntries(): Promise<EntryResponse> {
    let response: EntryResponse = {
        status: "",
        errorMessage: "",
        data: []
    }
    try {
        let entries = await invoke<string[]>('get_all_saved_flights');
        response.data = entries;
        response.status = "Success";
        if (entries.length == 0) {
            response.errorMessage = "No saved logs found";
        }
    } catch (error) {
        response.status = "Failed";
        response.errorMessage = "An error occured when loading saved files";
    } finally {
        return response;
    }
}

/** Saved flights with file size + modified time for the library view. */
export async function loadEntryDetails(): Promise<DetailsResponse> {
    try {
        const data = await invoke<SavedFlight[]>('get_saved_flight_details');
        return { ok: true, errorMessage: "", data };
    } catch (error) {
        return { ok: false, errorMessage: String(error || "An error occurred when loading saved files"), data: [] };
    }
}

export async function getDataPath(): Promise<string> {
    return await invoke<string>('get_data_path');
}

/** Opens a folder picker and saves the choice. Returns the new path, or null if cancelled. */
export async function chooseDataFolder(): Promise<string | null> {
    const path = await open({ multiple: false, directory: true });
    if (path == null) return null;
    await invoke('update_data_path', { path });
    libraryEvents.version++;
    return path;
}

export function fileNameOf(path: string | undefined): string {
    return path?.split(/[\\/]/).at(-1) ?? "";
}

export function extensionOf(path: string | undefined): string {
    const name = fileNameOf(path);
    const dot = name.lastIndexOf(".");
    return dot === -1 ? "" : name.slice(dot + 1).toLowerCase();
}

export function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    const units = ["KB", "MB", "GB"];
    let v = bytes / 1024, i = 0;
    while (v >= 1024 && i < units.length - 1) { v /= 1024; i++; }
    return `${v < 10 ? v.toFixed(1) : Math.round(v)} ${units[i]}`;
}
