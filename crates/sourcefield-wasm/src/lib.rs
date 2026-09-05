use wasm_bindgen::prelude::*;

const EPSILON: f32 = 0.000_1;

/// Deterministic force-field simulation used by the interactive GitHub Pages view.
///
/// The native generator owns the canonical graph and layout. This WebAssembly
/// module only adds small, bounded motion around those anchors so the browser
/// view stays faithful to the generated SVG.
#[wasm_bindgen]
pub struct Simulator {
    positions: Vec<f32>,
    velocities: Vec<f32>,
    anchors: Vec<f32>,
    edge_pairs: Vec<u32>,
    edge_weights: Vec<f32>,
    seed: u32,
    elapsed: f32,
    width: f32,
    height: f32,
}

#[wasm_bindgen]
impl Simulator {
    /// Creates a field bounded by the canonical canvas dimensions.
    ///
    /// # Errors
    /// Returns an error for non-finite coordinates, malformed edges, or an invalid canvas.
    #[wasm_bindgen(constructor)]
    pub fn new(
        positions: Vec<f32>,
        anchors: Vec<f32>,
        edge_pairs: Vec<u32>,
        edge_weights: Vec<f32>,
        seed: u32,
        width: f32,
        height: f32,
    ) -> Result<Simulator, JsValue> {
        validate_input(
            &positions,
            &anchors,
            &edge_pairs,
            &edge_weights,
            width,
            height,
        )
        .map_err(JsValue::from_str)?;

        Ok(Self {
            velocities: vec![0.0; positions.len()],
            positions,
            anchors,
            edge_pairs,
            edge_weights,
            seed: seed.max(1),
            elapsed: 0.0,
            width,
            height,
        })
    }

    /// Advances the simulation and returns flattened x/y coordinates.
    pub fn tick(&mut self, delta_seconds: f32) -> Vec<f32> {
        if !delta_seconds.is_finite() {
            return self.positions.clone();
        }

        let dt = delta_seconds.clamp(0.0, 0.05);
        if dt <= 0.0 {
            return self.positions.clone();
        }
        self.elapsed += dt;

        let count = self.positions.len() / 2;
        let mut forces = vec![0.0_f32; self.positions.len()];

        // Keep every node close to the deterministic native layout.
        for index in 0..count {
            let offset = index * 2;
            let anchor_strength = 7.5 + (index % 5) as f32 * 0.24;
            forces[offset] += (self.anchors[offset] - self.positions[offset]) * anchor_strength;
            forces[offset + 1] +=
                (self.anchors[offset + 1] - self.positions[offset + 1]) * anchor_strength;

            // Tiny seeded phase offsets prevent a mechanically uniform motion.
            let phase = seeded_fraction(self.seed, index as u32) * core::f32::consts::TAU;
            forces[offset] += (self.elapsed * 0.37 + phase).sin() * 0.78;
            forces[offset + 1] += (self.elapsed * 0.29 + phase * 1.37).cos() * 0.62;
        }

        // Mild pairwise repulsion. The graph is intentionally small, so O(n^2)
        // keeps the implementation dependency-free and deterministic.
        for left in 0..count {
            for right in (left + 1)..count {
                let lx = self.positions[left * 2];
                let ly = self.positions[left * 2 + 1];
                let rx = self.positions[right * 2];
                let ry = self.positions[right * 2 + 1];
                let dx = lx - rx;
                let dy = ly - ry;
                let distance_squared = (dx * dx + dy * dy).max(36.0);
                let distance = distance_squared.sqrt();
                let force = 34.0 / distance_squared;
                let fx = dx / distance * force;
                let fy = dy / distance * force;
                forces[left * 2] += fx;
                forces[left * 2 + 1] += fy;
                forces[right * 2] -= fx;
                forces[right * 2 + 1] -= fy;
            }
        }

        // Edges pull related nodes toward their original anchor distance rather
        // than toward an arbitrary fixed length.
        for edge_index in 0..self.edge_weights.len() {
            let left = self.edge_pairs[edge_index * 2] as usize;
            let right = self.edge_pairs[edge_index * 2 + 1] as usize;
            if left >= count || right >= count || left == right {
                continue;
            }

            let lx = self.positions[left * 2];
            let ly = self.positions[left * 2 + 1];
            let rx = self.positions[right * 2];
            let ry = self.positions[right * 2 + 1];
            let dx = rx - lx;
            let dy = ry - ly;
            let distance = (dx * dx + dy * dy).sqrt().max(EPSILON);

            let adx = self.anchors[right * 2] - self.anchors[left * 2];
            let ady = self.anchors[right * 2 + 1] - self.anchors[left * 2 + 1];
            let target = (adx * adx + ady * ady).sqrt().max(28.0);
            let strength = 0.42 + self.edge_weights[edge_index].clamp(0.0, 1.0) * 1.15;
            let displacement = (distance - target) * strength;
            let fx = dx / distance * displacement;
            let fy = dy / distance * displacement;

            forces[left * 2] += fx;
            forces[left * 2 + 1] += fy;
            forces[right * 2] -= fx;
            forces[right * 2 + 1] -= fy;
        }

        let damping = 0.88_f32.powf(dt * 60.0);
        for index in 0..count {
            let offset = index * 2;
            self.velocities[offset] = (self.velocities[offset] + forces[offset] * dt) * damping;
            self.velocities[offset + 1] =
                (self.velocities[offset + 1] + forces[offset + 1] * dt) * damping;

            self.positions[offset] += self.velocities[offset] * dt;
            self.positions[offset + 1] += self.velocities[offset + 1] * dt;

            // Canonical anchors use the declared canvas; fixed legacy dimensions clipped V3 nodes.
            self.positions[offset] = self.positions[offset].clamp(0.0, self.width);
            self.positions[offset + 1] = self.positions[offset + 1].clamp(0.0, self.height);
        }

        self.positions.clone()
    }

    /// Returns a copy of flattened x/y coordinates.
    pub fn positions(&self) -> Vec<f32> {
        self.positions.clone()
    }

    /// Restores anchors and clears velocity and elapsed animation time.
    pub fn reset(&mut self) -> Vec<f32> {
        self.positions.clone_from(&self.anchors);
        self.velocities.fill(0.0);
        self.elapsed = 0.0;
        self.positions.clone()
    }

    /// Returns the number of nodes.
    pub fn len(&self) -> usize {
        self.positions.len() / 2
    }

    /// Reports whether the field contains no nodes.
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
}

/// Keep validation pure so malformed browser inputs can be exercised on native targets.
fn validate_input(
    positions: &[f32],
    anchors: &[f32],
    edge_pairs: &[u32],
    edge_weights: &[f32],
    width: f32,
    height: f32,
) -> Result<(), &'static str> {
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return Err("canvas dimensions must be finite and positive");
    }

    if positions.is_empty()
        || !positions.len().is_multiple_of(2)
        || positions.len() != anchors.len()
    {
        return Err("positions and anchors must contain matching x/y pairs");
    }

    if positions
        .iter()
        .chain(anchors)
        .enumerate()
        .any(|(i, value)| {
            !value.is_finite() || *value < 0.0 || *value > if i % 2 == 0 { width } else { height }
        })
    {
        return Err("coordinates must be finite and inside the canvas");
    }

    if !edge_pairs.len().is_multiple_of(2) || edge_weights.len() != edge_pairs.len() / 2 {
        return Err("edges and weights must contain matching pairs");
    }

    if edge_pairs
        .iter()
        .any(|index| *index as usize >= positions.len() / 2)
        || edge_weights
            .iter()
            .any(|weight| !weight.is_finite() || !(0.0..=1.0).contains(weight))
    {
        return Err("edge indices and weights must be valid");
    }

    Ok(())
}

/// Derives repeatable phase offsets without an external random-number source.
fn seeded_fraction(seed: u32, index: u32) -> f32 {
    let mut value = seed ^ index.wrapping_mul(0x9E37_79B9);
    value ^= value << 13;
    value ^= value >> 17;
    value ^= value << 5;
    (value % 10_000) as f32 / 10_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulation_is_bounded_and_deterministic() {
        let positions = vec![100.0, 100.0, 200.0, 100.0];
        let anchors = positions.clone();
        let edges = vec![0, 1];
        let weights = vec![0.8];
        let mut left = Simulator::new(
            positions.clone(),
            anchors.clone(),
            edges.clone(),
            weights.clone(),
            42,
            1800.0,
            1680.0,
        )
        .unwrap();
        let mut right =
            Simulator::new(positions, anchors, edges, weights, 42, 1800.0, 1680.0).unwrap();

        assert_eq!(left.tick(0.016), right.tick(0.016));
        assert_eq!(left.len(), 2);
    }

    #[test]
    fn invalid_inputs_are_rejected_before_crossing_the_wasm_boundary() {
        let positions = [100.0, 100.0];

        let valid = validate_input(&positions, &positions, &[], &[], 1800.0, 1680.0);
        let nonfinite = validate_input(&[f32::NAN, 0.0], &positions, &[], &[], 1800.0, 1680.0);
        let invalid_edge = validate_input(&positions, &positions, &[0, 1], &[0.8], 1800.0, 1680.0);
        let invalid_canvas = validate_input(&positions, &positions, &[], &[], -1.0, 1680.0);

        assert!(valid.is_ok());
        assert!(nonfinite.is_err());
        assert!(invalid_edge.is_err());
        assert!(invalid_canvas.is_err());
    }

    #[test]
    fn long_running_motion_respects_custom_bounds_and_reset() {
        let anchors = vec![0.0, 0.0, 2200.0, 1900.0];
        let mut simulator = Simulator::new(
            anchors.clone(),
            anchors.clone(),
            vec![0, 1],
            vec![1.0],
            17,
            2200.0,
            1900.0,
        )
        .unwrap();

        for _ in 0..10_000 {
            let positions = simulator.tick(0.05);

            for pair in positions.as_chunks::<2>().0 {
                assert!((0.0..=2200.0).contains(&pair[0]));
                assert!((0.0..=1900.0).contains(&pair[1]));
            }
        }

        let before_invalid_tick = simulator.positions();
        let invalid_tick = simulator.tick(f32::NAN);
        let reset = simulator.reset();

        assert_eq!(invalid_tick, before_invalid_tick);
        assert_eq!(reset, anchors);
    }
}
