import { invoke } from '@tauri-apps/api/core';

type EntryResponse = {
    status: string,
    errorMessage: string,
    data: string[],
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