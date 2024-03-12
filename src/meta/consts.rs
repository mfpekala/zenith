/// Size of the window's width
pub const WINDOW_WIDTH: usize = 800;
/// Size of the window's height
pub const WINDOW_HEIGHT: usize = 800;

/// Number of pixels to show in screen width (should divide WINDOW_WIDTH)
pub const SCREEN_WIDTH: usize = 160;
/// Number of pixels to show in screen height (should divide WINDOW_HEIGHT)
pub const SCREEN_HEIGHT: usize = 160;

/// Kinda cursed if it's not this
pub const PIXEL_SIZE: usize = 1;

/// How many collisions can be resolved during a single frame? Caps memory usage of collision mechanism.
pub const MAX_COLLISIONS_PER_FRAME: usize = 16;
