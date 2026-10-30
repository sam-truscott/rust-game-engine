//! Shortest paths through a room's walkable area, found over the visibility graph of its corners.

use macroquad::math::Vec2;

use crate::room::Room;

const EPS: f32 = 1e-3;
const START: usize = 0;
const GOAL: usize = 1;
const FIRST_CORNER: usize = 2;

/// Which corners can see each other, computed once since a room's geometry never changes.
pub(crate) fn corner_visibility(room: &Room) -> Vec<Vec<bool>> {
    let corners = room.vertices();
    let count = corners.len();
    let mut visible = vec![vec![false; count]; count];
    for i in 0..count {
        for j in (i + 1)..count {
            let sees = is_visible(room, corners[i], corners[j]);
            visible[i][j] = sees;
            visible[j][i] = sees;
        }
    }
    visible
}

/// Returns the waypoints from start (exclusive) to goal (inclusive), or `None` if the goal is unreachable.
pub fn find(room: &Room, start: Vec2, goal: Vec2) -> Option<Vec<Vec2>> {
    let mut nodes = vec![start, goal];
    nodes.extend_from_slice(room.vertices());

    let count = nodes.len();
    let mut distance = vec![f32::MAX; count];
    distance[START] = 0.0;
    let mut previous = vec![usize::MAX; count];
    let mut done = vec![false; count];

    loop {
        let u = (0..count)
            .filter(|&i| !done[i] && distance[i] < f32::MAX)
            .min_by(|&a, &b| distance[a].total_cmp(&distance[b]))?;
        if u == GOAL {
            break;
        }
        done[u] = true;
        for v in 0..count {
            if done[v] {
                continue;
            }
            let candidate = distance[u] + nodes[u].distance(nodes[v]);
            if candidate < distance[v] && nodes_visible(room, &nodes, u, v) {
                distance[v] = candidate;
                previous[v] = u;
            }
        }
    }

    let mut path = Vec::new();
    let mut node = GOAL;
    while node != START {
        path.push(nodes[node]);
        node = previous[node];
    }
    path.reverse();
    Some(path)
}

fn nodes_visible(room: &Room, nodes: &[Vec2], u: usize, v: usize) -> bool {
    if u >= FIRST_CORNER && v >= FIRST_CORNER {
        return room.corners_visible(u - FIRST_CORNER, v - FIRST_CORNER);
    }
    is_visible(room, nodes[u], nodes[v])
}

/// True if the straight segment from `a` to `b` stays within the walkable area.
pub fn is_visible(room: &Room, a: Vec2, b: Vec2) -> bool {
    if a.distance_squared(b) < EPS * EPS {
        return true;
    }
    if room
        .edges()
        .iter()
        .any(|edge| crosses_properly(a, b, edge.a, edge.b))
    {
        return false;
    }

    // Corners lying on the segment split it, since each piece may be on a different side of the boundary.
    let direction = b - a;
    let length2 = direction.length_squared();
    let mut splits = vec![0.0, 1.0];
    for &vertex in room.vertices() {
        let t = (vertex - a).dot(direction) / length2;
        if t > EPS && t < 1.0 - EPS && (a + direction * t).distance_squared(vertex) < EPS * EPS {
            splits.push(t);
        }
    }
    splits.sort_by(f32::total_cmp);
    splits
        .windows(2)
        .all(|pair| room.is_walkable(a + direction * ((pair[0] + pair[1]) / 2.0)))
}

fn crosses_properly(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> bool {
    let d1 = orientation(a, b, c);
    let d2 = orientation(a, b, d);
    let d3 = orientation(c, d, a);
    let d4 = orientation(c, d, b);
    ((d1 > EPS && d2 < -EPS) || (d1 < -EPS && d2 > EPS))
        && ((d3 > EPS && d4 < -EPS) || (d3 < -EPS && d4 > EPS))
}

fn orientation(a: Vec2, b: Vec2, c: Vec2) -> f32 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}
