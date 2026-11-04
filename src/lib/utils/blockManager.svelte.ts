import { invoke } from "@tauri-apps/api/core";
import type { FlightData } from "./telemetryDisplay";
import { csvToFrames, snapshotToFrame, type Frame } from "./frames";

let filePath: string = $state('');

let firstBlock: Frame[] = $state([]);
let secondBlock: Frame[] = $state([]);
let usingFirst: boolean = $state(false);

let displayedLineIndex = $state(0);
let block_size = $derived(get_current_block().length - 1)
let linesUntilRefresh = $derived(
    block_size >= 0 ? Math.floor(block_size * 0.9) : 0
)

let reachedEnd: boolean = $derived(
    (block_size) >= 0 &&
    (displayedLineIndex >= block_size) &&
    (usingFirst ? secondBlock.length == 0 : firstBlock.length == 0)
);

export async function init_manager(path: string) {
    reset_manager();           // clears blocks + the tick clock from any previous file
    filePath = path;
    await load_new_block();
    usingFirst = !usingFirst;
}

export function reset_manager() {
    filePath = '';
    firstBlock = [];
    secondBlock = [];
    usingFirst = false;
    displayedLineIndex = 0;
}

function get_current_block(): Frame[] {
    return usingFirst ? firstBlock : secondBlock;
}

export function get_current_frame(): Frame | undefined {
    return get_current_block()[displayedLineIndex];
}

function get_next_frame(): Frame | undefined {
    if (reachedEnd) return undefined;
    if (displayedLineIndex == block_size) {
        return (usingFirst ? secondBlock : firstBlock)[0];
    }
    return get_current_block()[displayedLineIndex + 1];
}

export function get_current_frame_and_increment(): Frame | undefined {
    const frame = get_current_frame();
    increment_counter();
    return frame;
}

export function get_next_frame_and_increment(): Frame | undefined {
    const frame = get_next_frame();
    increment_counter();
    console.log("Getting the next frame and incrementing!");
    return frame;
}

function increment_counter() {
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

const MAX_WAIT_MS = 10_000;
const DEFAULT_WAIT_MS = 100;
export function determine_waiting_period(): number {
    const cur = get_current_frame(), next = get_next_frame();
    if (cur?.timeMs == null || next?.timeMs == null) return DEFAULT_WAIT_MS;
    const diff = next.timeMs - cur.timeMs;
    
    return diff < 0 ? 100 : Math.min(diff, MAX_WAIT_MS);
}

async function load_new_block() {
    // Pick the target buffer BEFORE awaiting. usingFirst can flip while the
    // request is in flight, which would write the block over the one on screen.
    const loadIntoFirst = !usingFirst;

    const res = await invoke<FlightData>('load_flight_block', { filePath });

    console.log(res);

    let frames: Frame[];
    if (res.kind === 'table') {
        const [headers = [], ...rows] = res.data;       // row 0 of every block is the header
        frames = csvToFrames(headers, rows);
    } else {
        frames = res.data.snapshots.map(snapshotToFrame);
    }

    console.log(frames);

    if (loadIntoFirst) firstBlock = frames; else secondBlock = frames;
}