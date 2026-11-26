export type CameraMode = "chase" | "side" | "top" | "ground" | "orbit" | "free";

export const CAMERA_OPTIONS: { value: CameraMode; label: string; hint: string }[] = [
    { value: "chase",  label: "Chase",        hint: "Follows behind and above the rocket" },
    { value: "side",   label: "Side",         hint: "Side-on view, best for reading pitch" },
    { value: "top",    label: "Top down",     hint: "Looking straight down the nose" },
    { value: "ground", label: "Ground",       hint: "Spectator on the field, tracking the rocket" },
    { value: "orbit",  label: "Orbit",        hint: "Slowly circles the rocket" },
    { value: "free",   label: "Free look",    hint: "Drag to rotate, scroll to zoom" },
];

export function cameraLabel(mode: CameraMode): string {
    return CAMERA_OPTIONS.find((c) => c.value === mode)?.label ?? mode;
}
