import { invoke } from "@tauri-apps/api/core";

let filePath: string = $state('');

let firstBlock: string[][] = $state([]);
let secondBlock: string[][] = $state([]);
let usingFirst: boolean = $state(false); 
let headers: string[] = $state([]);

let displayedLineIndex = $state(0);
let block_size = $derived(get_current_block().length - 1)
let linesUntilRefresh = $derived(
    block_size >= 0 ?
    Math.floor(block_size * 0.9) :
    0
)

let reachedEnd: boolean = $derived(
    // Block Size has initialized correctly
    (block_size) >= 0 &&
    // End of the current block
    (displayedLineIndex >= block_size) &&
    // Next block has no data
    (usingFirst ? secondBlock.length == 0 : firstBlock.length == 0)
);

export async function init_manager(path: string) {
    filePath = path;
    await load_new_block();
    usingFirst = !usingFirst;
}

export function get_headers(): string[] {
    return headers;
}

function get_current_block(): string[][] {
    return usingFirst ? firstBlock : secondBlock;
}

export function get_current_line(): string[] {
    return get_current_block()[displayedLineIndex];
}

function get_next_line(): string[] {
    if (reachedEnd) {
        return [];
    }
    if (displayedLineIndex == block_size) {
        return usingFirst ? secondBlock[0] : firstBlock[0];
    }

    return get_current_block()[displayedLineIndex + 1];
}

export function get_current_line_and_increment(): string[] {
    let line = get_current_line();
    increment_counter();
    return line;
}

export function get_next_line_and_increment(): string[] {
    let line = get_next_line();
    increment_counter();
    return line;
}

function increment_counter() {
    // console.log("Current Index: " + displayedLineIndex);
    // console.log("Lines Until Refresh : " + linesUntilRefresh);
    if (displayedLineIndex == linesUntilRefresh) {
        load_new_block();
    }

    if (displayedLineIndex == block_size) {
        usingFirst = !usingFirst;
        displayedLineIndex = 0;
    } else {
        displayedLineIndex++;
    }
}

export function determine_waiting_period(): number {
    // TODO - determine what other headers contain valid time data
    // TODO - find a better way to do this rather than manually listing valid headers
    const DEFAULT_WAIT = 100;
    const SECOND_HEADERS = ["unixtime", "relseconds"];
    const MS_HEADERS = ["relms", "currenttimemillis", "unixms"]

    let firstLine = get_current_line();
    let secondLine = get_next_line();

    let usingSeconds: boolean = false;

    let timeIndex = headers.findIndex((head) => {
        if (SECOND_HEADERS.includes(head.toLowerCase())) {
            usingSeconds = true;
            return true;
        } else {
            return MS_HEADERS.includes(head.toLowerCase());
        }
    })

    if (timeIndex == -1) {
        return DEFAULT_WAIT;
    }

    let difference = Number(secondLine[timeIndex]) - Number(firstLine[timeIndex]);
    return usingSeconds ? difference * 1000 : difference;
}

async function load_new_block() {
    console.log("Loading a new block!");
    // Block will always have headers in the 0th index,
    // The rest of the data will occupy the rest of the space
    let loadedBlock = await invoke<string[][]>('load_flight_block', {filePath: filePath});
    headers = loadedBlock[0];
    // A bit counter-intuitive, but we want to avoid updating the data we're currently displaying
    if (usingFirst) {
        secondBlock = loadedBlock.slice(1);
    } else {
        firstBlock = loadedBlock.slice(1);
    }
}