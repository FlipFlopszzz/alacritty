// Fork: scrollbar ported from TetraTheta/alacritty-scrollbar (PR #8494 lineage).
// Rect is defined locally to avoid a glutin dependency.
use std::time::{Duration, Instant};

use alacritty_terminal::grid::Scroll;

use crate::config::ui_config::{Scrollbar as ScrollbarConfig, ScrollbarMode};

use super::SizeInfo;

/// Keeps track of when the scrollbar should be visible or fading.
#[derive(Debug, Clone, PartialEq)]
pub struct Scrollbar {
    config: ScrollbarConfig,
    /// Display offset, that was last used to draw the scrollbar.
    display_offset: usize,
    /// Total lines, that was last used to draw the scrollbar.
    total_lines: usize,
    last_change: Option<Instant>,
    drag_state: Option<DragState>,
    /// Fork: the mouse hovers the scrollbar trough.
    hover: bool,
    /// Fork: opacity as of the last frame, so hover-highlight changes can be
    /// damaged — otherwise the new pixels never reach the screen.
    last_drawn_opacity: Option<f32>,
}

impl From<&ScrollbarConfig> for Scrollbar {
    fn from(value: &ScrollbarConfig) -> Self {
        Scrollbar {
            config: value.clone(),
            display_offset: 0,
            total_lines: 0,
            last_change: None,
            drag_state: None,
            hover: false,
            last_drawn_opacity: None,
        }
    }
}

impl Scrollbar {
    /// Fork: whether the mouse hovers the scrollbar.
    pub fn set_hover(&mut self, hover: bool) {
        self.hover = hover;
    }

    /// Fork: current hover state.
    pub fn is_hovered(&self) -> bool {
        self.hover
    }

    /// Fork: record the opacity drawn this frame and report whether it
    /// changed since the previous one.
    pub fn take_opacity_change(&mut self, opacity: f32) -> bool {
        let changed = self.last_drawn_opacity != Some(opacity);
        self.last_drawn_opacity = Some(opacity);
        changed
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ScrollbarState {
    Show {
        opacity: f32,
    },
    WaitForFading {
        opacity: f32,
        remaining_duration: Duration,
    },
    Fading {
        opacity: f32,
    },
    /// `has_damage` - If the scrollbar was previously viisible, we need to draw a damage rect.
    Invisible {
        has_damage: bool,
    },
}

impl Scrollbar {
    pub fn update_config(&mut self, config: &ScrollbarConfig) {
        self.config = config.clone();
    }

    /// Returns whether the scrollbar position or height needs an update.
    pub fn update(&mut self, display_offset: usize, total_lines: usize) -> bool {
        if self.display_offset != display_offset {
            self.display_offset = display_offset;
            self.total_lines = total_lines;
            self.last_change = Some(Instant::now());
            true
        } else if self.total_lines != total_lines {
            self.total_lines = total_lines;
            self.last_change = Some(Instant::now());
            true
        } else {
            false
        }
    }

    fn last_change_time(&self) -> Option<Instant> {
        self.last_change
    }

    fn clear_change_time(&mut self) {
        self.last_change = None;
    }

    pub fn is_visible(&self, display_size: SizeInfo) -> bool {
        match self.config.mode {
            ScrollbarMode::Never => false,
            ScrollbarMode::Fading => {
                self.is_dragging()
                    || self.total_lines > display_size.screen_lines
                        && self.last_change_time().is_some()
            },
            ScrollbarMode::Always => true,
        }
    }

    pub fn intensity(&mut self, display_size: SizeInfo) -> ScrollbarState {
        match self.config.mode {
            ScrollbarMode::Never => ScrollbarState::Invisible { has_damage: false },
            ScrollbarMode::Fading => {
                if self.total_lines <= display_size.screen_lines {
                    return ScrollbarState::Invisible { has_damage: false };
                }
                if self.is_dragging() {
                    self.last_change = Some(Instant::now());
                }
                if let Some(last_scroll) = self.last_change_time() {
                    let timeout = (Instant::now() - last_scroll).as_millis();
                    let fade_wait = (self.config.duration as f32 * 0.8).floor() as u128;
                    let fade_time = self.config.duration as u128 - fade_wait;
                    if timeout <= fade_wait {
                        let remaining = fade_wait - timeout;
                        let opacity = self.config.opacity.as_f32();
                        let remaining_duration = Duration::from_millis(remaining as u64);
                        ScrollbarState::WaitForFading { opacity, remaining_duration }
                    } else {
                        let current_fade_time = timeout - fade_wait;
                        if current_fade_time < fade_time {
                            // Fading progress from 0.0 to 1.0.
                            let fading_progress = current_fade_time as f32 / fade_time as f32;
                            let opacity = (1.0 - fading_progress) * self.config.opacity.as_f32();
                            ScrollbarState::Fading { opacity }
                        } else {
                            self.clear_change_time();
                            ScrollbarState::Invisible { has_damage: true }
                        }
                    }
                } else {
                    ScrollbarState::Invisible { has_damage: false }
                }
            },
            ScrollbarMode::Always => {
                // Fork: full opacity while hovering or dragging the handle.
                let opacity = if self.hover || self.drag_state.is_some() {
                    1.0
                } else {
                    self.config.opacity.as_f32()
                };
                ScrollbarState::Show { opacity }
            },
        }
    }

    pub fn bg_rect(&self, display_size: SizeInfo) -> Rect {
        let background_area_height: f32 = display_size.height;

        let scrollbar_width = display_size.cell_width;
        let x = display_size.width - scrollbar_width;
        Rect {
            x: x.floor() as i32,
            y: 0,
            width: scrollbar_width.ceil() as i32,
            height: background_area_height.ceil() as i32,
        }
    }

    /// Fork: handle rectangle in top-based screen coordinates.
    pub fn handle_rect(&self, bg_rect: Rect, display_size: SizeInfo) -> Rect {
        let height_fraction = display_size.screen_lines as f32 / self.total_lines as f32;
        let scrollbar_height =
            (height_fraction * bg_rect.height as f32).max(2. * display_size.cell_height);

        // `display_offset` 0 (newest content) puts the handle at the BOTTOM.
        let y_progress = if self.total_lines <= display_size.screen_lines {
            0.0
        } else {
            self.display_offset as f32 / (self.total_lines - display_size.screen_lines) as f32
        };
        let y = bg_rect.y as f32 + (1.0 - y_progress) * (bg_rect.height as f32 - scrollbar_height);

        Rect {
            x: bg_rect.x,
            y: y.floor() as i32,
            width: bg_rect.width,
            height: scrollbar_height.ceil() as i32,
        }
    }

    /// Fork: whether the mouse is over the scrollbar handle.
    pub fn contains_mouse_pos(
        &self,
        display_size: SizeInfo,
        mouse_x: usize,
        mouse_y: usize,
    ) -> bool {
        if !self.is_visible(display_size) {
            return false;
        }

        let bg_rect = self.bg_rect(display_size);
        let handle = self.handle_rect(bg_rect, display_size);
        let mouse_x = mouse_x as f32;
        let mouse_y = mouse_y as f32;

        if !(handle.x as f32..(handle.x + handle.width) as f32).contains(&mouse_x) {
            return false;
        }

        (handle.y as f32..(handle.y + handle.height) as f32).contains(&mouse_y)
    }

    /// Fork: whether the mouse is anywhere over the scrollbar trough.
    pub fn contains_track_pos(
        &self,
        display_size: SizeInfo,
        mouse_x: usize,
        mouse_y: usize,
    ) -> bool {
        if !self.is_visible(display_size) {
            return false;
        }

        let bg_rect = self.bg_rect(display_size);
        let mouse_x = mouse_x as f32;
        let mouse_y = mouse_y as f32;

        if !(bg_rect.x as f32..(bg_rect.x + bg_rect.width) as f32).contains(&mouse_x) {
            return false;
        }

        (bg_rect.y as f32..(bg_rect.y + bg_rect.height) as f32).contains(&mouse_y)
    }

    /// Fork: clicking the trough jumps the view so that the clicked position
    /// becomes the top of the viewport. Returns the scroll delta to apply.
    pub fn click_jump(
        &mut self,
        display_size: SizeInfo,
        mouse_x: usize,
        mouse_y: usize,
    ) -> Option<Scroll> {
        if !self.contains_track_pos(display_size, mouse_x, mouse_y) {
            return None;
        }

        let bg_rect = self.bg_rect(display_size);
        // Fraction measured from the BOTTOM of the trough: display_offset 0
        // means the viewport is at the bottom (newest content), so clicking
        // the top of the trough jumps to the oldest lines.
        let fraction =
            (1.0 - (mouse_y as f32 - bg_rect.y as f32) / bg_rect.height as f32).clamp(0.0, 1.0);
        let max_offset = (self.total_lines - display_size.screen_lines) as f32;
        let target_offset = (fraction * max_offset).round() as usize;

        let delta = target_offset as i32 - self.display_offset as i32;
        if delta == 0 {
            return None;
        }
        // Update the internal state immediately so hover/drag feedback matches
        // the jump before the next redraw.
        self.display_offset = target_offset;
        self.last_change = Some(Instant::now());

        Some(Scroll::Delta(delta))
    }

    pub fn try_start_drag(
        &mut self,
        display_size: SizeInfo,
        mouse_x: usize,
        mouse_y: usize,
    ) -> bool {
        if !self.contains_mouse_pos(display_size, mouse_x, mouse_y) {
            return false;
        }

        let bg_rect = self.bg_rect(display_size);
        let rect = self.handle_rect(bg_rect, display_size);

        if bg_rect.height == rect.height || self.total_lines <= display_size.screen_lines {
            self.drag_state =
                Some(DragState { cells_per_dragged_pixel: 0.0, accumulated_cells: 0. });
            return true;
        }

        // Amount of pixels, you have to drag over, to scroll from top to bottom.
        let total_pixel_scroll = bg_rect.height - rect.height;
        let total_lines_to_scroll = self.total_lines - display_size.screen_lines;
        let cells_per_dragged_pixel = total_lines_to_scroll as f32 / total_pixel_scroll as f32;
        self.drag_state = Some(DragState { cells_per_dragged_pixel, accumulated_cells: 0. });

        true
    }

    pub fn is_dragging(&self) -> bool {
        self.drag_state.is_some()
    }

    pub fn stop_dragging(&mut self) {
        self.drag_state = None;
    }

    #[must_use = "The actual scroll is not applied but returned and has to be applied by the \
                  callside"]
    pub fn apply_mouse_delta(&mut self, mouse_y_delta_in_pixel: f32) -> Option<Scroll> {
        if let Some(drag_state) = self.drag_state.as_mut() {
            drag_state.accumulated_cells +=
                mouse_y_delta_in_pixel * drag_state.cells_per_dragged_pixel;
            let cells = drag_state.accumulated_cells as i32; // round towards zero
            if cells == 0 {
                None
            } else {
                drag_state.accumulated_cells -= cells as f32;
                Some(Scroll::Delta(-cells))
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct DragState {
    cells_per_dragged_pixel: f32,
    accumulated_cells: f32,
}

/// Integer rectangle used for scrollbar geometry.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
