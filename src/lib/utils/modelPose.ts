const AXES = ["X", "Y", "Z"] as const;

export const POSE_CAL = {
    longAxis: 1 as 0 | 1 | 2,    // 0=X 1=Y 2=Z: which axis runs nose-to-tail in the GLB. Y is a GUESS until the log says otherwise
    sign: [1, 1, 1] as number[], // flip an entry to -1 if that direction looks mirrored
};

/** [roll, pitch, yaw] in degrees -> three.js radians plus an Euler order that keeps roll body-fixed. */
export function toModelPose(pose: number[]) {
    const long = POSE_CAL.longAxis;
    const [a, b] = [0, 1, 2].filter((i) => i !== long);        // the two "lean" axes
    const rad = (deg: number, i: number) => ((deg * Math.PI) / 180) * POSE_CAL.sign[i];

    const out = [0, 0, 0];
    out[long] = rad(pose[0], 0);   // roll: spin about the nose-tail axis
    out[a]    = rad(pose[1], 1);   // pitch (tilt from vertical)
    out[b]    = rad(pose[2], 2);   // yaw

    // roll axis LAST, so the spin stays about the rocket's own axis (Y long -> "XZY")
    const order = [a, b, long].map((i) => AXES[i]).join("") as
        "XYZ" | "XZY" | "YXZ" | "YZX" | "ZXY" | "ZYX";
    return { x: out[0], y: out[1], z: out[2], order };
}