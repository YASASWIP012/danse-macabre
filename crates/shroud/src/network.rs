//! The shuffle network. A triangle of adjacent 2x2 switches can route any permutation, because it
//! is exactly the set of comparisons bubble sort makes.

use alloc::vec::Vec;

/// The wire pairs of the network for `n` cards, in order.
pub fn switches(n: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for round in 0..n.saturating_sub(1) {
        for wire in 0..n - 1 - round {
            out.push((wire, wire + 1));
        }
    }
    out
}

/// The switch settings that carry card `i` to position `destinations[i]`. Sampling `destinations`
/// uniformly gives a uniform shuffle, which setting each switch by coin flip would not.
pub fn route(destinations: &[usize]) -> Vec<bool> {
    let mut wires = destinations.to_vec();
    switches(destinations.len())
        .into_iter()
        .map(|(a, b)| {
            let cross = wires[a] > wires[b];
            if cross {
                wires.swap(a, b);
            }
            cross
        })
        .collect()
}
