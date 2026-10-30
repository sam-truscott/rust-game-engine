/// Which way a character faces on screen; `Front` is towards the viewer and `Back` is away.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Front,
    Back,
    Left,
    Right,
}
