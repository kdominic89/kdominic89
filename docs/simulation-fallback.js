/** Deterministic JavaScript fallback, explicitly distinct from the Rust/WASM engine. */
export class Simulator {
    /** Copy caller-owned buffers and reject invalid coordinates before animation starts. */
    constructor(positions, anchors, edgePairs, edgeWeights, seed = 1, width = 1800, height = 1680) {
        if (!Number.isFinite(width) || !Number.isFinite(height) || width <= 0 || height <= 0 ||
            positions.length === 0 || positions.length % 2 || anchors.length !== positions.length ||
            edgePairs.length % 2 || edgeWeights.length !== edgePairs.length / 2) {
            throw new RangeError('Invalid simulation shape');
        }

        for (const values of [positions, anchors]) {
            if (values.some((value, index) => !Number.isFinite(value) || value < 0 ||
                value > (index % 2 ? height : width))) {
                throw new RangeError('Coordinates must be inside the canvas');
            }
        }

        if (edgePairs.some(index => !Number.isInteger(index) || index < 0 || index >= positions.length / 2) ||
            edgeWeights.some(weight => !Number.isFinite(weight) || weight < 0 || weight > 1)) {
            throw new RangeError('Invalid edge');
        }

        this.positionData = Float32Array.from(positions);
        this.anchorData = Float32Array.from(anchors);
        this.velocityData = new Float32Array(positions.length);
        this.seed = seed || 1;
        this.width = width;
        this.height = height;
        this.elapsed = 0;
    }

    /** Advance bounded decorative motion; invalid time deltas leave the field unchanged. */
    tick(deltaSeconds) {
        if (!Number.isFinite(deltaSeconds) || deltaSeconds <= 0) return this.positions();

        const dt = Math.min(.05, deltaSeconds);
        this.elapsed += dt;
        const damping = Math.pow(.88, dt * 60);

        for (let index = 0; index < this.positionData.length / 2; index++) {
            const offset = index * 2;
            const phase = seeded(this.seed, index) * Math.PI * 2;
            const fx = (this.anchorData[offset] - this.positionData[offset]) * 7.5 +
                Math.sin(this.elapsed * .37 + phase) * .78;
            const fy = (this.anchorData[offset + 1] - this.positionData[offset + 1]) * 7.5 +
                Math.cos(this.elapsed * .29 + phase * 1.37) * .62;

            this.velocityData[offset] = (this.velocityData[offset] + fx * dt) * damping;
            this.velocityData[offset + 1] = (this.velocityData[offset + 1] + fy * dt) * damping;
            this.positionData[offset] = Math.max(0, Math.min(this.width,
                this.positionData[offset] + this.velocityData[offset] * dt));
            this.positionData[offset + 1] = Math.max(0, Math.min(this.height,
                this.positionData[offset + 1] + this.velocityData[offset + 1] * dt));
        }

        return this.positions();
    }

    /** Return a defensive coordinate copy. */
    positions() {
        return this.positionData.slice();
    }

    /** Restore the original anchors and clear accumulated motion. */
    reset() {
        this.positionData.set(this.anchorData);
        this.velocityData.fill(0);
        this.elapsed = 0;

        return this.positions();
    }

    /** Return the number of simulated nodes. */
    len() {
        return this.positionData.length / 2;
    }

    /** Report whether the field contains no nodes. */
    is_empty() {
        return this.positionData.length === 0;
    }
}

/** Match the Rust phase seed without relying on random browser state. */
function seeded(seed, index) {
    let value = (seed ^ Math.imul(index, 0x9e3779b9)) >>> 0;
    value ^= value << 13;
    value ^= value >>> 17;
    value ^= value << 5;

    return (value >>> 0) % 10000 / 10000;
}
