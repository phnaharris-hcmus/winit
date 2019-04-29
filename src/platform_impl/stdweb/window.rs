use dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize};
use event::{Event, WindowEvent};
use icon::Icon;
use monitor::{MonitorHandle as RootMH};
use window::{CreationError, MouseCursor, WindowAttributes, WindowId as RootWI};
use super::{EventLoopWindowTarget, register};
use std::collections::VecDeque;
use std::collections::vec_deque::IntoIter as VecDequeIter;
use std::cell::RefCell;
use stdweb::{
    traits::*,
    unstable::TryInto
};
use stdweb::web::{
    document, window,
    html_element::CanvasElement,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MonitorHandle;

impl MonitorHandle {
    pub fn get_hidpi_factor(&self) -> f64 {
        1.0
    }

    pub fn get_position(&self) -> PhysicalPosition {
        unimplemented!();
    }

    pub fn get_dimensions(&self) -> PhysicalSize {
        unimplemented!();
    }

    pub fn get_name(&self) -> Option<String> {
        unimplemented!();
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlatformSpecificWindowBuilderAttributes;

impl WindowId {
    pub unsafe fn dummy() -> WindowId {
        WindowId
    }
}

pub struct Window {
    pub(crate) canvas: CanvasElement,
    pub(crate) redraw: Box<dyn Fn()>,
    previous_pointer: RefCell<&'static str>,
    position: RefCell<LogicalPosition>,
}

impl Window {
    pub fn new<T>(target: &EventLoopWindowTarget<T>, attr: WindowAttributes,
                  _: PlatformSpecificWindowBuilderAttributes) -> Result<Self, CreationError> {
        let element = document()
            .create_element("canvas")
            .map_err(|_| CreationError::OsError("Failed to create canvas element".to_owned()))?;
        let canvas: CanvasElement = element.try_into()
            .map_err(|_| CreationError::OsError("Failed to create canvas element".to_owned()))?;
        document().body()
            .ok_or_else(|| CreationError::OsError("Failed to find body node".to_owned()))?
            .append_child(&canvas);

        register(&target.runner, &canvas);

        let runner = target.runner.clone();
        let redraw = Box::new(move || window().request_animation_frame(|| runner.send_event(Event::WindowEvent {
            window_id: RootWI(WindowId),
            event: WindowEvent::RedrawRequested
        })));

        let window = Window {
            canvas,
            redraw,
            previous_pointer: RefCell::new("auto"),
            position: RefCell::new(LogicalPosition {
                x: 0.0,
                y: 0.0
            })
        };

        if let Some(dimensions) = attr.dimensions {
            window.set_inner_size(dimensions);
        } else {
            window.set_inner_size(LogicalSize {
                width: 1024.0,
                height: 768.0,
            })
        }
        window.set_min_dimensions(attr.min_dimensions);
        window.set_max_dimensions(attr.max_dimensions);
        window.set_resizable(attr.resizable);
        window.set_title(&attr.title);
        window.set_maximized(attr.maximized);
        if attr.visible {
            window.show();
        } else {
            window.hide();
        }
        //window.set_transparent(attr.transparent);
        window.set_decorations(attr.decorations);
        window.set_always_on_top(attr.always_on_top);
        window.set_window_icon(attr.window_icon);

        Ok(window)
    }

    pub fn set_title(&self, title: &str) {
        document().set_title(title);
    }

    pub fn show(&self) {
        // Intentionally a no-op
    }

    pub fn hide(&self) {
        // Intentionally a no-op
    }

    pub fn request_redraw(&self) {
        (self.redraw)();
    }

    pub fn get_position(&self) -> Option<LogicalPosition> {
        let bounds = self.canvas.get_bounding_client_rect();
        Some(LogicalPosition {
            x: bounds.get_x(),
            y: bounds.get_y(),
        })
    }

    pub fn get_inner_position(&self) -> Option<LogicalPosition> {
        Some(*self.position.borrow())
    }

    pub fn set_position(&self, position: LogicalPosition) {
        *self.position.borrow_mut() = position;
        self.canvas.set_attribute("position", "fixed")
            .expect("Setting the position for the canvas");
        self.canvas.set_attribute("left", &position.x.to_string())
            .expect("Setting the position for the canvas");
        self.canvas.set_attribute("top", &position.y.to_string())
            .expect("Setting the position for the canvas");
    }

    #[inline]
    pub fn get_inner_size(&self) -> Option<LogicalSize> {
        Some(LogicalSize {
            width: self.canvas.width() as f64,
            height: self.canvas.height() as f64
        })
    }

    #[inline]
    pub fn get_outer_size(&self) -> Option<LogicalSize> {
        Some(LogicalSize {
            width: self.canvas.width() as f64,
            height: self.canvas.height() as f64
        })
    }

    #[inline]
    pub fn set_inner_size(&self, size: LogicalSize) {
        self.canvas.set_width(size.width as u32);
        self.canvas.set_height(size.height as u32);
    }

    #[inline]
    pub fn set_min_dimensions(&self, _dimensions: Option<LogicalSize>) {
        // Intentionally a no-op: users can't resize canvas elements
    }

    #[inline]
    pub fn set_max_dimensions(&self, _dimensions: Option<LogicalSize>) {
        // Intentionally a no-op: users can't resize canvas elements
    }

    #[inline]
    pub fn set_resizable(&self, _resizable: bool) {
        // Intentionally a no-op: users can't resize canvas elements
    }

    #[inline]
    pub fn get_hidpi_factor(&self) -> f64 {
        1.0
    }

    #[inline]
    pub fn set_cursor(&self, cursor: MouseCursor) {
        let text = match cursor {
            MouseCursor::Default => "auto",
            MouseCursor::Crosshair => "crosshair",
            MouseCursor::Hand => "pointer",
            MouseCursor::Arrow => "default",
            MouseCursor::Move => "move",
            MouseCursor::Text => "text",
            MouseCursor::Wait => "wait",
            MouseCursor::Help => "help",
            MouseCursor::Progress => "progress",

            MouseCursor::NotAllowed => "not-allowed",
            MouseCursor::ContextMenu => "context-menu",
            MouseCursor::Cell => "cell",
            MouseCursor::VerticalText => "vertical-text",
            MouseCursor::Alias => "alias",
            MouseCursor::Copy => "copy",
            MouseCursor::NoDrop => "no-drop",
            MouseCursor::Grab => "grab",
            MouseCursor::Grabbing => "grabbing",
            MouseCursor::AllScroll => "all-scroll",
            MouseCursor::ZoomIn => "zoom-in",
            MouseCursor::ZoomOut => "zoom-out",

            MouseCursor::EResize => "e-resize",
            MouseCursor::NResize => "n-resize",
            MouseCursor::NeResize => "ne-resize",
            MouseCursor::NwResize => "nw-resize",
            MouseCursor::SResize => "s-resize",
            MouseCursor::SeResize => "se-resize",
            MouseCursor::SwResize => "sw-resize",
            MouseCursor::WResize => "w-resize",
            MouseCursor::EwResize => "ew-resize",
            MouseCursor::NsResize => "ns-resize",
            MouseCursor::NeswResize => "nesw-resize",
            MouseCursor::NwseResize => "nwse-resize",
            MouseCursor::ColResize => "col-resize",
            MouseCursor::RowResize => "row-resize",
        };
        *self.previous_pointer.borrow_mut() = text;
        self.canvas.set_attribute("cursor", text)
            .expect("Setting the cursor on the canvas");
    }

    #[inline]
    pub fn set_cursor_position(&self, _position: LogicalPosition) -> Result<(), String> {
        // TODO: pointer capture
        Ok(())
    }

    #[inline]
    pub fn grab_cursor(&self, _grab: bool) -> Result<(), String> {
        // TODO: pointer capture
        Ok(())
    }

    #[inline]
    pub fn hide_cursor(&self, hide: bool) {
        if hide {
            self.canvas.set_attribute("cursor", "none")
                .expect("Setting the cursor on the canvas");
        } else {
            self.canvas.set_attribute("cursor", *self.previous_pointer.borrow())
                .expect("Setting the cursor on the canvas");
        }
    }

    #[inline]
    pub fn set_maximized(&self, _maximized: bool) {
        // TODO: should there be a maximization / fullscreen API?
    }

    #[inline]
    pub fn set_fullscreen(&self, _monitor: Option<RootMH>) {
        // TODO: should there be a maximization / fullscreen API?
    }

    #[inline]
    pub fn set_decorations(&self, _decorations: bool) {
        // Intentionally a no-op, no canvas decorations
    }

    #[inline]
    pub fn set_always_on_top(&self, _always_on_top: bool) {
        // Intentionally a no-op, no window ordering
    }

    #[inline]
    pub fn set_window_icon(&self, _window_icon: Option<Icon>) {
        // Currently an intentional no-op
    }

    #[inline]
    pub fn set_ime_spot(&self, _position: LogicalPosition) {
        // TODO: what is this?
    }

    #[inline]
    pub fn get_current_monitor(&self) -> RootMH {
        RootMH {
            inner: MonitorHandle
        }
    }

    #[inline]
    pub fn get_available_monitors(&self) -> VecDequeIter<MonitorHandle> {
        VecDeque::new().into_iter()
    }

    #[inline]
    pub fn get_primary_monitor(&self) -> MonitorHandle {
        MonitorHandle
    }

    #[inline]
    pub fn id(&self) -> WindowId {
        // TODO ?
        unsafe { WindowId::dummy() }
    }
}
