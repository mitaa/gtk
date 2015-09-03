// Copyright 2015, The Rust-GNOME Project Developers.
// See the COPYRIGHT file at the top-level directory of this distribution.
// Licensed under the MIT license, see the LICENSE file or <http://opensource.org/licenses/MIT>

//use std::boxed::into_raw;
use std::mem::transmute;

use traits::FFIWidget;
use glib::signal::connect;
use glib::translate::*;
use glib::{FFIGObject, ParamSpec};

use glib_ffi::gboolean;
use ffi::{
    GtkAdjustment,
    GtkTreeSelection,
    GtkTreeViewColumn,
    GtkTreeIter,
    GtkTreePath,
};
use libc;
use gdk::{
    EventAny,
    EventButton,
    EventConfigure,
    EventCrossing,
    EventExpose,
    EventFocus,
    EventGrabBroken,
    EventKey,
    EventMotion,
    EventProperty,
    EventProximity,
    EventScroll,
    EventWindowState,
    Screen,
};
use cairo::{Context, RectangleInt};

use {
    Adjustment,
    Button,
    ComboBox,
    DeleteType,
    Dialog,
    DirectionType,
    Entry,
    MovementStep,
    Range,
    ScrollType,
    SpinButton,
    StateFlags,
    TextDirection,
    ToolButton,
    Tooltip,
    TreeIter,
    TreePath,
    TreeSelection,
    TreeView,
    TreeViewColumn,
    Widget,
    WidgetHelpType,
};

/// Whether to propagate the signal to other handlers
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Inhibit(pub bool);

impl ToGlib for Inhibit {
    type GlibType = gboolean;

    #[inline]
    fn to_glib(&self) -> gboolean {
        self.0.to_glib()
    }
}

// libstd stability workaround
unsafe fn into_raw<T>(b: Box<T>) -> *mut T { transmute(b) }


macro_rules! signal_trait(
    ($trait_name:ident, $this_ty:ty, $($func_name:ident(_$(,$arg_ty:ty)*) $(-> $out_ty:ty),*),+) => (
        pub trait $trait_name {
            $(
                fn $func_name<F: Fn($this_ty, $($arg_ty),*) $(-> $out_ty)*>(&self, f: F) -> u64;
            )+
        }
    )
);

macro_rules! signal_impl (
    ($this_translates:ident, $the_trait:ty, $this_ty: ty, $this_ffi_ty: ty, $($name: expr, $func_name: ident
    (_$(,$arg_name: ident:($arg_ffi_ty: ty > $translate: ident > $arg_ty: ty))*) -> ($out_ty:ty > $out_translate:ident > $out_ffi_ty:ty)),+) => (
        impl $the_trait for $this_ty {
            $(
            signal_method!(
                $this_translates,
                $name,
                $func_name (this: (*mut $this_ffi_ty > $this_translates > $this_ty)
                    $(, $arg_name: ($arg_ffi_ty > $translate > $arg_ty))*
                    ) -> ($out_ty > $out_translate > $out_ffi_ty));
            )+
        }
    )
);

macro_rules! signal_method (
    ($this_translates:ident, $name: expr, $func_name: ident ($($arg:ident: ($arg_ffi_ty:ty > $translate:ident > $arg_ty:ty)),*) -> ($out_ty:ty > $out_translate:ident > $out_ffi_ty:ty)) => (
        fn $func_name<F: Fn($($arg_ty),*) -> $out_ty + 'static>(&self, f: F) -> u64 {
            extern "C" fn trampoline($($arg: $arg_ffi_ty,)* f: &Box<Fn($($arg_ty),*) -> $out_ty + 'static>) -> $out_ffi_ty {
                $out_translate!(
                    $out_ty,
                    f($($translate!($arg_ty, $arg),)*)
                )
            }
            use std::mem::transmute;
            use glib::signal::connect;

            unsafe {
                let f: Box<Box<Fn($($arg_ty),*) -> $out_ty + 'static>> = Box::new(Box::new(f));
                // FIXME
                let this = self;
                connect($this_translates!(this) as *mut _, $name,
                    transmute(trampoline), into_raw(f) as *mut _)
            }
        }
    )
);

macro_rules! widget{
    ($ty:ty, $e:expr) => (FFIWidget::wrap_widget($e as *mut _));
    ($e:expr) => (($e.unwrap_widget()));
}

macro_rules! gobject{
    ($ty:ty, $e:expr) => (<$ty>::wrap_pointer($e));
    ($e:expr) => ($e.unwrap_pointer());
}

macro_rules! void { ($ty:ty, $e:expr) => ($e;) }
macro_rules! echo { ($ty:ty, $e:expr) => ($e) }
macro_rules! to_glib { ($ty:ty, $e:expr) => ($e.to_glib()) }
macro_rules! from_glib { ($ty:ty, $e:expr) => (from_glib($e)) }

// can't do <&mut TreeIter>::wrap_pointer...
macro_rules! translate_treeiter { ($ty:ty, $e:expr) => ( &mut TreeIter::wrap_pointer($e)) }


pub trait WidgetSignals {
    fn connect_notify<F: Fn(Widget, &ParamSpec) + 'static>(&self, f: F) -> u64;
    fn connect_accel_closures_changed<F: Fn(Widget) + 'static>(&self, f: F) -> u64;
    fn connect_button_press_event<F: Fn(Widget, &EventButton) -> Inhibit + 'static>(&self, f: F) -> u64;
    fn connect_button_release_event<F: Fn(Widget, &EventButton) -> Inhibit + 'static>(&self, f: F) -> u64;
    fn connect_can_activate_accel<F: Fn(Widget, u64) -> bool + 'static>(&self, f: F) -> u64;
    fn connect_child_notify<F: Fn(Widget, &ParamSpec) + 'static>(&self, f: F) -> u64;
    fn connect_composited_changed<F: Fn(Widget) + 'static>(&self, f: F) -> u64;
    fn connect_configure_event<F: Fn(Widget, &EventConfigure) -> Inhibit + 'static>(&self, f: F) -> u64;
    fn connect_damage_event<F: Fn(Widget, &EventExpose) -> Inhibit + 'static>(&self, f: F) -> u64;
    fn connect_delete_event<F: Fn(Widget, &EventAny) -> Inhibit + 'static>(&self, f: F) -> u64;
    fn connect_destroy<F: Fn(Widget) + 'static>(&self, f: F) -> u64;
    fn connect_destroy_event<F: Fn(Widget, &EventAny) -> Inhibit + 'static>(&self, f: F) -> u64;
    fn connect_direction_changed<F: Fn(Widget, TextDirection) + 'static>(&self, f: F) -> u64;
    fn connect_draw<F: Fn(Widget, Context) -> Inhibit + 'static>(&self, f: F) -> u64;
    fn connect_enter_notify_event<F: Fn(Widget, &EventCrossing) -> Inhibit + 'static>(&self, f: F) -> u64;
    fn connect_event<F: Fn(Widget, &EventAny) -> Inhibit + 'static>(&self, f: F) -> u64;
    fn connect_event_after<F: Fn(Widget, &EventAny) + 'static>(&self, f: F) -> u64;
    fn connect_focus<F: Fn(Widget, DirectionType) -> Inhibit + 'static>(&self, f: F) -> u64;
    fn connect_focus_in_event<F: Fn(Widget, &EventFocus) -> Inhibit + 'static>(&self, f: F) -> u64;
    fn connect_focus_out_event<F: Fn(Widget, &EventFocus) -> Inhibit + 'static>(&self, f: F) -> u64;
    fn connect_grab_broken_event<F: Fn(Widget, &EventGrabBroken) -> Inhibit + 'static>(&self, f: F) -> u64;
    fn connect_grab_focus<F: Fn(Widget) + 'static>(&self, f: F) -> u64;
    fn connect_grab_notify<F: Fn(Widget, bool) + 'static>(&self, f: F) -> u64;
    fn connect_hide<F: Fn(Widget) + 'static>(&self, f: F) -> u64;
    fn connect_key_press_event<F: Fn(Widget, &EventKey) -> Inhibit + 'static>(&self, f: F) -> u64;
    fn connect_key_release_event<F: Fn(Widget, &EventKey) -> Inhibit + 'static>(&self, f: F) -> u64;
    fn connect_keynav_failed<F: Fn(Widget, DirectionType) -> Inhibit + 'static>(&self, f: F) -> u64;
    fn connect_map<F: Fn(Widget) + 'static>(&self, f: F) -> u64;
    fn connect_map_event<F: Fn(Widget, &EventAny) -> Inhibit + 'static>(&self, f: F) -> u64;
    fn connect_mnemonic_activate<F: Fn(Widget, bool) -> Inhibit + 'static>(&self, f: F) -> u64;
    fn connect_motion_notify_event<F: Fn(Widget, &EventMotion) -> Inhibit + 'static>(&self, f: F) -> u64;
    fn connect_move_focus<F: Fn(Widget, DirectionType) + 'static>(&self, f: F) -> u64;
    fn connect_popup_menu<F: Fn(Widget) -> bool + 'static>(&self, f: F) -> u64;
    fn connect_property_notify_event<F: Fn(Widget, &EventProperty) -> Inhibit + 'static>(&self, f: F) -> u64;
    fn connect_proximity_in_event<F: Fn(Widget, &EventProximity) -> Inhibit + 'static>(&self, f: F) -> u64;
    fn connect_proximity_out_event<F: Fn(Widget, &EventProximity) -> Inhibit + 'static>(&self, f: F) -> u64;
    fn connect_query_tooltip<F: Fn(Widget, i32, i32, bool, Tooltip) -> bool + 'static>(&self, f: F) -> u64;
    fn connect_realize<F: Fn(Widget) + 'static>(&self, f: F) -> u64;
    fn connect_screen_changed<F: Fn(Widget, Screen) + 'static>(&self, f: F) -> u64;
    fn connect_scroll_event<F: Fn(Widget, &EventScroll) -> Inhibit + 'static>(&self, f: F) -> u64;
    fn connect_show<F: Fn(Widget) + 'static>(&self, f: F) -> u64;
    fn connect_show_help<F: Fn(Widget, WidgetHelpType) -> bool + 'static>(&self, f: F) -> u64;
    fn connect_size_allocate<F: Fn(Widget, &RectangleInt) + 'static>(&self, f: F) -> u64;
    fn connect_state_flags_changed<F: Fn(Widget, StateFlags) + 'static>(&self, f: F) -> u64;
    fn connect_style_updated<F: Fn(Widget) + 'static>(&self, f: F) -> u64;
    fn connect_touch_event<F: Fn(Widget, &EventAny) -> Inhibit + 'static>(&self, f: F) -> u64;
    fn connect_unmap<F: Fn(Widget) + 'static>(&self, f: F) -> u64;
    fn connect_unmap_event<F: Fn(Widget, &EventAny) -> Inhibit + 'static>(&self, f: F) -> u64;
    fn connect_unrealize<F: Fn(Widget) + 'static>(&self, f: F) -> u64;
    fn connect_window_state_event<F: Fn(Widget, &EventWindowState) -> Inhibit + 'static>(&self, f: F) -> u64;
}

mod widget {
    use super::into_raw;
    use std::mem::transmute;
    use libc::{c_int, c_uint};
    use glib::{ParamSpec};
    use glib::signal::connect;
    use glib::translate::*;
    use gdk::{
        EventAny, EventButton, EventConfigure, EventCrossing, EventExpose, EventFocus,
        EventGrabBroken, EventKey, EventMotion, EventProperty, EventProximity, EventScroll,
        EventWindowState, Screen,
    };
    use cairo_ffi::cairo_t;
    use cairo::{Context, RectangleInt};
    use traits::{FFIWidget, WidgetTrait};
    use gdk_ffi::GdkScreen;
    use glib_ffi::gboolean;
    use ffi::{GtkWidget, GtkTooltip};
    use {Widget, DirectionType, StateFlags, TextDirection, Tooltip, WidgetHelpType};
    use super::Inhibit;

    impl<T: FFIWidget + WidgetTrait> super::WidgetSignals for T {
        // this is a GObject signal actually
        fn connect_notify<F: Fn(Widget, &ParamSpec) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, &ParamSpec) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "notify",
                    transmute(notify_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_accel_closures_changed<F: Fn(Widget) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "accel-closures-changed",
                    transmute(void_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_button_press_event<F: Fn(Widget, &EventButton) -> Inhibit + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, &EventButton) -> Inhibit + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "button-press-event",
                    transmute(event_button_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_button_release_event<F: Fn(Widget, &EventButton) -> Inhibit + 'static>(&self, f: F)
                -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, &EventButton) -> Inhibit + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "button-release-event",
                    transmute(event_button_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_can_activate_accel<F: Fn(Widget, u64) -> bool + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, u64) -> bool + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "can-activate-accel",
                    transmute(accel_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_child_notify<F: Fn(Widget, &ParamSpec) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, &ParamSpec) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "child-notify",
                    transmute(notify_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_composited_changed<F: Fn(Widget) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "composited-changed",
                    transmute(void_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_configure_event<F: Fn(Widget, &EventConfigure) -> Inhibit + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, &EventConfigure) -> Inhibit + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "configure-event",
                    transmute(event_configure_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_damage_event<F: Fn(Widget, &EventExpose) -> Inhibit + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, &EventExpose) -> Inhibit + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "damage-event",
                    transmute(event_expose_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_delete_event<F: Fn(Widget, &EventAny) -> Inhibit + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, &EventAny) -> Inhibit + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "delete-event",
                    transmute(event_any_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_destroy<F: Fn(Widget) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "destroy",
                    transmute(void_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_destroy_event<F: Fn(Widget, &EventAny) -> Inhibit + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, &EventAny) -> Inhibit + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "destroy-event",
                    transmute(event_any_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_direction_changed<F: Fn(Widget, TextDirection) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, TextDirection) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "direction-changed",
                    transmute(text_direction_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_draw<F: Fn(Widget, Context) -> Inhibit + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, Context) -> Inhibit + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "draw",
                    transmute(draw_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_enter_notify_event<F: Fn(Widget, &EventCrossing) -> Inhibit + 'static>(&self, f: F)
                -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, &EventCrossing) -> Inhibit + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "enter-notify-event",
                    transmute(event_crossing_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_event<F: Fn(Widget, &EventAny) -> Inhibit + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, &EventAny) -> Inhibit + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "event",
                    transmute(event_any_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_event_after<F: Fn(Widget, &EventAny) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, &EventAny) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "event-after",
                    transmute(event_any_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_focus<F: Fn(Widget, DirectionType) -> Inhibit + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, DirectionType) -> Inhibit + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "focus",
                    transmute(direction_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_focus_in_event<F: Fn(Widget, &EventFocus) -> Inhibit + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, &EventFocus) -> Inhibit + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "focus-in-event",
                    transmute(event_focus_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_focus_out_event<F: Fn(Widget, &EventFocus) -> Inhibit + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, &EventFocus) -> Inhibit + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "focus-out-event",
                    transmute(event_focus_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_grab_broken_event<F: Fn(Widget, &EventGrabBroken) -> Inhibit + 'static>(&self, f: F)
                -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, &EventGrabBroken) -> Inhibit + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "grab-broken-event",
                    transmute(event_grab_broken_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_grab_focus<F: Fn(Widget) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "grab-focus",
                    transmute(void_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_grab_notify<F: Fn(Widget, bool) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, bool) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "grab-notify",
                    transmute(grab_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_hide<F: Fn(Widget) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "hide",
                    transmute(void_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_keynav_failed<F: Fn(Widget, DirectionType) -> Inhibit + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, DirectionType) -> Inhibit + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "keynav-failed",
                    transmute(direction_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_key_press_event<F: Fn(Widget, &EventKey) -> Inhibit + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, &EventKey) -> Inhibit + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "key-press-event",
                    transmute(event_key_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_key_release_event<F: Fn(Widget, &EventKey) -> Inhibit + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, &EventKey) -> Inhibit + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "key-release-event",
                    transmute(event_key_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_map<F: Fn(Widget) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "map",
                    transmute(void_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_map_event<F: Fn(Widget, &EventAny) -> Inhibit + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, &EventAny) -> Inhibit + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "map-event",
                    transmute(event_any_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_mnemonic_activate<F: Fn(Widget, bool) -> Inhibit + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, bool) -> Inhibit + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "mnemonic-activate",
                    transmute(mnemonic_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_move_focus<F: Fn(Widget, DirectionType) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, DirectionType) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "move-focus",
                    transmute(direction_void_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_motion_notify_event<F: Fn(Widget, &EventMotion) -> Inhibit + 'static>(&self, f: F)
                -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, &EventMotion) -> Inhibit + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "motion-notify-event",
                    transmute(event_motion_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_property_notify_event<F: Fn(Widget, &EventProperty) -> Inhibit + 'static>(&self, f: F)
                -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, &EventProperty) -> Inhibit + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "property-notify-event",
                    transmute(event_property_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_proximity_in_event<F: Fn(Widget, &EventProximity) -> Inhibit + 'static>(&self, f: F)
                -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, &EventProximity) -> Inhibit + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "proximity-in-event",
                    transmute(event_proximity_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_proximity_out_event<F: Fn(Widget, &EventProximity) -> Inhibit + 'static>(&self, f: F)
                -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, &EventProximity) -> Inhibit + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "proximity-out-event",
                    transmute(event_proximity_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_popup_menu<F: Fn(Widget) -> bool + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget) -> bool + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "popup-menu",
                    transmute(bool_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_query_tooltip<F: Fn(Widget, i32, i32, bool, Tooltip) -> bool + 'static>(&self, f: F)
                -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, i32, i32, bool, Tooltip) -> bool + 'static>> =
                    Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "query-tooltip",
                    transmute(query_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_realize<F: Fn(Widget) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "realize",
                    transmute(void_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_screen_changed<F: Fn(Widget, Screen) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, Screen) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "screen-changed",
                    transmute(screen_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_scroll_event<F: Fn(Widget, &EventScroll) -> Inhibit + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, &EventScroll) -> Inhibit + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "scroll-event",
                    transmute(event_scroll_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_show<F: Fn(Widget) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "show",
                    transmute(void_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_show_help<F: Fn(Widget, WidgetHelpType) -> bool + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, WidgetHelpType) -> bool + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "show-help",
                    transmute(help_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_size_allocate<F: Fn(Widget, &RectangleInt) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, &RectangleInt) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "size-allocate",
                    transmute(rectangle_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_state_flags_changed<F: Fn(Widget, StateFlags) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, StateFlags) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "state-flags-changed",
                    transmute(state_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_style_updated<F: Fn(Widget) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "style-updated",
                    transmute(void_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_touch_event<F: Fn(Widget, &EventAny) -> Inhibit + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, &EventAny) -> Inhibit + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "touch-event",
                    transmute(event_any_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_unmap<F: Fn(Widget) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "unmap",
                    transmute(void_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_unmap_event<F: Fn(Widget, &EventAny) -> Inhibit + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, &EventAny) -> Inhibit + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "unmap-event",
                    transmute(event_any_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_unrealize<F: Fn(Widget) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "unrealize",
                    transmute(void_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_window_state_event<F: Fn(Widget, &EventWindowState) -> Inhibit + 'static>(&self, f: F)
                -> u64 {
            unsafe {
                let f: Box<Box<Fn(Widget, &EventWindowState) -> Inhibit + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "window-state-event",
                    transmute(event_window_state_trampoline), into_raw(f) as *mut _)
            }
        }

    }

    extern "C" fn void_trampoline(this: *mut GtkWidget, f: &Box<Fn(Widget) + 'static>) {
        f(FFIWidget::wrap_widget(this));
    }

    extern "C" fn bool_trampoline(this: *mut GtkWidget, f: &Box<Fn(Widget) -> bool + 'static>) -> gboolean {
        f(FFIWidget::wrap_widget(this)).to_glib()
    }

    extern "C" fn accel_trampoline(this: *mut GtkWidget, signal_id: c_uint,
            f: &Box<Fn(Widget, u64) -> bool + 'static>) -> gboolean {
        f(FFIWidget::wrap_widget(this), signal_id as u64).to_glib()
    }

    extern "C" fn draw_trampoline(this: *mut GtkWidget, cr: *mut cairo_t,
            f: &Box<Fn(Widget, Context) -> Inhibit + 'static>) -> gboolean {
        unsafe { f(FFIWidget::wrap_widget(this), from_glib_none(cr)).to_glib() }
    }

    extern "C" fn event_any_trampoline(this: *mut GtkWidget, event: *mut EventAny,
            f: &Box<Fn(Widget, &EventAny) -> Inhibit + 'static>) -> gboolean {
        unsafe { f(FFIWidget::wrap_widget(this), transmute(event)).to_glib() }
    }

    extern "C" fn event_button_trampoline(this: *mut GtkWidget, event: *mut EventAny,
            f: &Box<Fn(Widget, &EventButton) -> Inhibit + 'static>) -> gboolean {
        unsafe { f(FFIWidget::wrap_widget(this), transmute(event)).to_glib() }
    }

    extern "C" fn event_configure_trampoline(this: *mut GtkWidget, event: *mut EventAny,
            f: &Box<Fn(Widget, &EventConfigure) -> Inhibit + 'static>) -> gboolean {
        unsafe { f(FFIWidget::wrap_widget(this), transmute(event)).to_glib() }
    }

    extern "C" fn event_crossing_trampoline(this: *mut GtkWidget, event: *mut EventAny,
            f: &Box<Fn(Widget, &EventCrossing) -> Inhibit + 'static>) -> gboolean {
        unsafe { f(FFIWidget::wrap_widget(this), transmute(event)).to_glib() }
    }

    extern "C" fn event_expose_trampoline(this: *mut GtkWidget, event: *mut EventAny,
            f: &Box<Fn(Widget, &EventExpose) -> Inhibit + 'static>) -> gboolean {
        unsafe { f(FFIWidget::wrap_widget(this), transmute(event)).to_glib() }
    }

    extern "C" fn event_focus_trampoline(this: *mut GtkWidget, event: *mut EventAny,
            f: &Box<Fn(Widget, &EventFocus) -> Inhibit + 'static>) -> gboolean {
        unsafe { f(FFIWidget::wrap_widget(this), transmute(event)).to_glib() }
    }

    extern "C" fn event_grab_broken_trampoline(this: *mut GtkWidget, event: *mut EventAny,
            f: &Box<Fn(Widget, &EventGrabBroken) -> Inhibit + 'static>) -> gboolean {
        unsafe { f(FFIWidget::wrap_widget(this), transmute(event)).to_glib() }
    }

    extern "C" fn event_key_trampoline(this: *mut GtkWidget, event: *mut EventAny,
            f: &Box<Fn(Widget, &EventKey) -> Inhibit + 'static>) -> gboolean {
        unsafe { f(FFIWidget::wrap_widget(this), transmute(event)).to_glib() }
    }

    extern "C" fn event_motion_trampoline(this: *mut GtkWidget, event: *mut EventAny,
            f: &Box<Fn(Widget, &EventMotion) -> Inhibit + 'static>) -> gboolean {
        unsafe { f(FFIWidget::wrap_widget(this), transmute(event)).to_glib() }
    }

    extern "C" fn event_property_trampoline(this: *mut GtkWidget, event: *mut EventAny,
            f: &Box<Fn(Widget, &EventProperty) -> Inhibit + 'static>) -> gboolean {
        unsafe { f(FFIWidget::wrap_widget(this), transmute(event)).to_glib() }
    }

    extern "C" fn event_proximity_trampoline(this: *mut GtkWidget, event: *mut EventAny,
            f: &Box<Fn(Widget, &EventProximity) -> Inhibit + 'static>) -> gboolean {
        unsafe { f(FFIWidget::wrap_widget(this), transmute(event)).to_glib() }
    }

    extern "C" fn event_scroll_trampoline(this: *mut GtkWidget, event: *mut EventAny,
            f: &Box<Fn(Widget, &EventScroll) -> Inhibit + 'static>) -> gboolean {
        unsafe { f(FFIWidget::wrap_widget(this), transmute(event)).to_glib() }
    }

    extern "C" fn event_window_state_trampoline(this: *mut GtkWidget, event: *mut EventAny,
            f: &Box<Fn(Widget, &EventWindowState) -> Inhibit + 'static>) -> gboolean {
        unsafe { f(FFIWidget::wrap_widget(this), transmute(event)).to_glib() }
    }

    extern "C" fn direction_trampoline(this: *mut GtkWidget, direction: DirectionType,
            f: &Box<Fn(Widget, DirectionType) -> Inhibit + 'static>) -> gboolean {
        f(FFIWidget::wrap_widget(this), direction).to_glib()
    }

    extern "C" fn direction_void_trampoline(this: *mut GtkWidget, direction: DirectionType,
            f: &Box<Fn(Widget, DirectionType) + 'static>) {
        f(FFIWidget::wrap_widget(this), direction);
    }

    extern "C" fn grab_trampoline(this: *mut GtkWidget, was_grabbed: gboolean,
            f: &Box<Fn(Widget, bool) + 'static>) {
        f(FFIWidget::wrap_widget(this), from_glib(was_grabbed));
    }

    extern "C" fn help_trampoline(this: *mut GtkWidget, help_type: WidgetHelpType,
            f: &Box<Fn(Widget, WidgetHelpType) -> bool + 'static>) -> gboolean {
        f(FFIWidget::wrap_widget(this), help_type).to_glib()
    }

    extern "C" fn mnemonic_trampoline(this: *mut GtkWidget, arg1: gboolean,
            f: &Box<Fn(Widget, bool) -> Inhibit + 'static>) -> gboolean {
        f(FFIWidget::wrap_widget(this), from_glib(arg1)).to_glib()
    }

    extern "C" fn notify_trampoline(this: *mut GtkWidget, pspec: *mut ParamSpec,
            f: &Box<Fn(Widget, &ParamSpec) + 'static>) {
        unsafe { f(FFIWidget::wrap_widget(this), transmute(pspec)); }
    }

    extern "C" fn query_trampoline(this: *mut GtkWidget, x: c_int, y: c_int, keyboard: gboolean,
            _tooltip: *mut GtkTooltip, f: &Box<Fn(Widget, i32, i32, bool, Tooltip) -> bool + 'static>)
            -> gboolean {
        f(FFIWidget::wrap_widget(this), x, y, from_glib(keyboard), Tooltip).to_glib()
    }

    extern "C" fn rectangle_trampoline(this: *mut GtkWidget, allocation: *mut RectangleInt,
            f: &Box<Fn(Widget, &RectangleInt) + 'static>) {
        unsafe { f(FFIWidget::wrap_widget(this), transmute(allocation)); }
    }

    extern "C" fn state_trampoline(this: *mut GtkWidget, flags: StateFlags,
            f: &Box<Fn(Widget, StateFlags) + 'static>) {
        f(FFIWidget::wrap_widget(this), flags);
    }

    extern "C" fn screen_trampoline(this: *mut GtkWidget, screen: *mut GdkScreen,
            f: &Box<Fn(Widget, Screen) + 'static>) {
        unsafe { f(FFIWidget::wrap_widget(this), Screen::from_glib_none(screen)); }
    }

    extern "C" fn text_direction_trampoline(this: *mut GtkWidget, previous: TextDirection,
            f: &Box<Fn(Widget, TextDirection) + 'static>) {
        f(FFIWidget::wrap_widget(this), previous);
    }

}

pub trait EntrySignals {
    fn connect_activate<F: Fn(Entry) + 'static>(&self, f: F) -> u64;
    fn connect_backspace<F: Fn(Entry) + 'static>(&self, f: F) -> u64;
    fn connect_copy_clipboard<F: Fn(Entry) + 'static>(&self, f: F) -> u64;
    fn connect_cut_clipboard<F: Fn(Entry) + 'static>(&self, f: F) -> u64;
    fn connect_paste_clipboard<F: Fn(Entry) + 'static>(&self, f: F) -> u64;
    fn connect_toggle_overwrite<F: Fn(Entry) + 'static>(&self, f: F) -> u64;
    fn connect_delete_from_cursor<F: Fn(Entry, DeleteType, i32) + 'static>(&self, f: F) -> u64;
    fn connect_move_cursor<F: Fn(Entry, MovementStep, i32, bool) + 'static>(&self, f: F) -> u64;
    fn connect_insert_at_cursor<F: Fn(Entry, &str) + 'static>(&self, f: F) -> u64;
    fn connect_preedit_changed<F: Fn(Entry, &str) + 'static>(&self, f: F) -> u64;
}

mod entry {
    use super::into_raw;
    use std::mem::transmute;
    use std::str;
    use std::ffi::CStr;
    use glib::signal::connect;
    use libc::c_char;
    use traits::{FFIWidget, EntryTrait};
    use ffi::GtkEntry;
    use {Entry, DeleteType, MovementStep};

    impl<T: FFIWidget + EntryTrait> super::EntrySignals for T {
        fn connect_activate<F: Fn(Entry) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Entry) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "activate",
                    transmute(void_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_backspace<F: Fn(Entry) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Entry) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "backspace",
                    transmute(void_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_copy_clipboard<F: Fn(Entry) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Entry) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "copy_clipboard",
                    transmute(void_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_cut_clipboard<F: Fn(Entry) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Entry) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "cut_clipboard",
                    transmute(void_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_paste_clipboard<F: Fn(Entry) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Entry) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "paste_clipboard",
                    transmute(void_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_toggle_overwrite<F: Fn(Entry) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Entry) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "toggle_overwrite",
                    transmute(void_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_delete_from_cursor<F: Fn(Entry, DeleteType, i32) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Entry, DeleteType, i32) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "delete_from_cursor",
                    transmute(delete_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_move_cursor<F: Fn(Entry, MovementStep, i32, bool) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Entry, MovementStep, i32, bool) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "move_cursor",
                    transmute(move_cursor_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_insert_at_cursor<F: Fn(Entry, &str) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Entry, &str) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "insert_at_cursor",
                    transmute(string_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_preedit_changed<F: Fn(Entry, &str) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Entry, &str) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "preedit_changed",
                    transmute(string_trampoline), into_raw(f) as *mut _)
            }
        }
    }

    extern "C" fn void_trampoline(this: *mut GtkEntry, f: &Box<Fn(Entry) + 'static>) {
        f(FFIWidget::wrap_widget(this as *mut _));
    }

    extern "C" fn delete_trampoline(this: *mut GtkEntry, delete_type: DeleteType, count: i32,
                                    f: &Box<Fn(Entry, DeleteType, i32) + 'static>) {
        f(FFIWidget::wrap_widget(this as *mut _), delete_type, count);
    }

    extern "C" fn move_cursor_trampoline(this: *mut GtkEntry, step: MovementStep, count: i32,
                                         extend_selection: bool,
                                         f: &Box<Fn(Entry, MovementStep, i32, bool) + 'static>) {
        f(FFIWidget::wrap_widget(this as *mut _), step, count, extend_selection);
    }

    extern "C" fn string_trampoline(this: *mut GtkEntry, c_str: *const c_char,
                                    f: &Box<Fn(Entry, &str) + 'static>) {
        let buf = unsafe { CStr::from_ptr(c_str).to_bytes() };
        let string = str::from_utf8(buf).unwrap();
        f(FFIWidget::wrap_widget(this as *mut _), string);
    }
}

pub trait ButtonSignals {
    fn connect_activate<F: Fn(Button) + 'static>(&self, f: F) -> u64;
    fn connect_clicked<F: Fn(Button) + 'static>(&self, f: F) -> u64;
}

mod button {
    use super::into_raw;
    use std::mem::transmute;
    use glib::signal::connect;
    use traits::{FFIWidget, ButtonTrait};
    use ffi::GtkButton;
    use Button;

    impl<T: FFIWidget + ButtonTrait> super::ButtonSignals for T {
        fn connect_activate<F: Fn(Button) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Button) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "activate",
                    transmute(void_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_clicked<F: Fn(Button) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(Button) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "clicked",
                    transmute(void_trampoline), into_raw(f) as *mut _)
            }
        }
    }

    extern "C" fn void_trampoline(this: *mut GtkButton, f: &Box<Fn(Button) + 'static>) {
        f(FFIWidget::wrap_widget(this as *mut _));
    }
}

pub trait ComboBoxSignals {
    fn connect_changed<F: Fn(ComboBox) + 'static>(&self, f: F) -> u64;
    fn connect_move_active<F: Fn(ComboBox, ScrollType) + 'static>(&self, f: F) -> u64;
    fn connect_popdown<F: Fn(ComboBox) -> bool + 'static>(&self, f: F) -> u64;
    fn connect_popup<F: Fn(ComboBox) + 'static>(&self, f: F) -> u64;
}

mod combobox {
    use super::into_raw;
    use std::mem::transmute;
    use glib::signal::connect;
    use glib::translate::*;
    use glib_ffi::gboolean;
    use ffi::GtkComboBox;
    use traits::{FFIWidget, ComboBoxTrait};
    use {ComboBox, ScrollType};

    impl<T: FFIWidget + ComboBoxTrait> super::ComboBoxSignals for T {
        fn connect_changed<F: Fn(ComboBox) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(ComboBox) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "changed",
                    transmute(void_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_move_active<F: Fn(ComboBox, ScrollType) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(ComboBox, ScrollType) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "move-active",
                    transmute(move_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_popdown<F: Fn(ComboBox) -> bool + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(ComboBox) -> bool + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "popdown",
                    transmute(bool_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_popup<F: Fn(ComboBox) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(ComboBox) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "popup",
                    transmute(void_trampoline), into_raw(f) as *mut _)
            }
        }
    }

    extern "C" fn void_trampoline(this: *mut GtkComboBox, f: &Box<Fn(ComboBox) + 'static>) {
        f(FFIWidget::wrap_widget(this as *mut _));
    }

    extern "C" fn bool_trampoline(this: *mut GtkComboBox, f: &Box<Fn(ComboBox) -> bool + 'static>)
            -> gboolean {
        f(FFIWidget::wrap_widget(this as *mut _)).to_glib()
    }

    extern "C" fn move_trampoline(this: *mut GtkComboBox, scroll_type: ScrollType,
            f: &Box<Fn(ComboBox, ScrollType) + 'static>) {
        f(FFIWidget::wrap_widget(this as *mut _), scroll_type);
    }
}

pub trait ToolButtonSignals {
    fn connect_clicked<F: Fn(ToolButton) + 'static>(&self, f: F) -> u64;
}

mod tool_button {
    use super::into_raw;
    use std::mem::transmute;
    use glib::signal::connect;
    use traits::{FFIWidget, ToolButtonTrait};
    use ffi::GtkToolButton;
    use ToolButton;

    impl<T: FFIWidget + ToolButtonTrait> super::ToolButtonSignals for T {
        fn connect_clicked<F: Fn(ToolButton) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(ToolButton) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "clicked",
                    transmute(void_trampoline), into_raw(f) as *mut _)
            }
        }
    }

    extern "C" fn void_trampoline(this: *mut GtkToolButton, f: &Box<Fn(ToolButton) + 'static>) {
        f(FFIWidget::wrap_widget(this as *mut _));
    }
}

pub trait SpinButtonSignals {
    fn connect_value_changed<F: Fn(SpinButton) + 'static>(&self, f: F) -> u64;
    fn connect_wrapped<F: Fn(SpinButton) + 'static>(&self, f: F) -> u64;
}

mod spin_button {
    use super::into_raw;
    use std::mem::transmute;
    use glib::signal::connect;
    use traits::FFIWidget;
    use ffi::GtkSpinButton;
    use SpinButton;

    impl super::SpinButtonSignals for SpinButton {
        fn connect_value_changed<F: Fn(SpinButton) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(SpinButton) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "value-changed",
                    transmute(void_trampoline), into_raw(f) as *mut _)
            }
        }

        fn connect_wrapped<F: Fn(SpinButton) + 'static>(&self, f: F) -> u64 {
            unsafe {
                let f: Box<Box<Fn(SpinButton) + 'static>> = Box::new(Box::new(f));
                connect(self.unwrap_widget() as *mut _, "clicked",
                    transmute(void_trampoline), into_raw(f) as *mut _)
            }
        }
    }

    extern "C" fn void_trampoline(this: *mut GtkSpinButton, f: &Box<Fn(SpinButton) + 'static>) {
        f(FFIWidget::wrap_widget(this as *mut _));
    }
}

signal_trait!(DialogSignals, Dialog,
    connect_close(_),
    connect_response(_, i32)
);
signal_impl!(widget, DialogSignals, Dialog, ::ffi::GtkDialog,
    "close", connect_close(_) -> (() > void > ()),
    "response", connect_response(_, response: (libc::c_int > echo > i32)) -> (() > void > ())
);


signal_trait!(TreeViewSignals, TreeView,
    connect_columns_changed(_),
    connect_cursor_changed(_),
    connect_expand_collapse_cursor_row(_, bool, bool, bool) -> bool,
    connect_row_activated(_, TreePath, TreeViewColumn),
    connect_row_collapsed(_, &mut TreeIter, TreePath),
    connect_row_expanded(_, &mut TreeIter, TreePath),
    connect_select_all(_) -> bool,
    connect_select_cursor_parent(_) -> bool,
    connect_select_cursor_row(_, bool) -> bool,
    connect_start_interactive_search(_) -> bool,
    connect_test_collapse_row(_, &mut TreeIter, TreePath) -> bool,
    connect_test_expand_row(_, &mut TreeIter, TreePath) -> bool,
    connect_toggle_cursor_row(_) -> bool,
    connect_unselect_all(_) -> bool
);
signal_impl!(widget, TreeViewSignals, TreeView, ::ffi::GtkTreeView,
    "columns-changed", connect_columns_changed(_) -> (() > void > ()),
    "cursor-changed", connect_cursor_changed(_) -> (() > void > ()),
    "expand-collapse-cursor-row", connect_expand_collapse_cursor_row
        (_, arg1: (gboolean > from_glib > bool),
            arg2: (gboolean > from_glib > bool),
            arg3: (gboolean > from_glib > bool)) -> (bool > to_glib > gboolean),
    "row-activated", connect_row_activated
        (_, path: (*mut GtkTreePath > gobject > TreePath),
            column: (*mut GtkTreeViewColumn > gobject > TreeViewColumn)) -> (() > void > ()),
    "row-collapsed", connect_row_collapsed
        (_, iter: (*mut GtkTreeIter > translate_treeiter > &mut TreeIter),
            path: (*mut GtkTreePath > gobject > TreePath)) -> (() > void > ()),
    "row-expanded", connect_row_expanded
        (_, iter: (*mut GtkTreeIter > translate_treeiter > &mut TreeIter),
            path: (*mut GtkTreePath > gobject > TreePath)) -> (() > void > ()),
    "select-all", connect_select_all(_) -> (bool > to_glib > gboolean),
    "select-cursor-parent", connect_select_cursor_parent(_) -> (bool > to_glib > gboolean),
    "select-cursor-row", connect_select_cursor_row
        (_, arg1: (gboolean > from_glib > bool)) -> (bool > to_glib > gboolean),
    "start-interactive-search", connect_start_interactive_search(_) -> (bool > to_glib > gboolean),
    "test-collapse-row", connect_test_collapse_row
        (_, iter: (*mut GtkTreeIter > translate_treeiter > &mut TreeIter),
            path: (*mut GtkTreePath > gobject > TreePath)) -> (bool > to_glib > gboolean),
    "test-expand-row", connect_test_expand_row
        (_, iter: (*mut GtkTreeIter > translate_treeiter > &mut TreeIter),
            path: (*mut GtkTreePath > gobject > TreePath)) -> (bool > to_glib > gboolean),
    "toggle-cursor-row", connect_toggle_cursor_row(_) -> (bool > to_glib > gboolean),
    "unselect-all", connect_unselect_all(_) -> (bool > to_glib > gboolean)
);


signal_trait!(RangeSignals, Range,
    connect_adjust_bounds(_, f64),
    connect_change_value(_, ScrollType, f64) -> Inhibit,
    connect_move_slider(_, ScrollType),
    connect_value_changed(_)
);
signal_impl!(widget, RangeSignals, Range, ::ffi::GtkRange,
    "adjust-bounds", connect_adjust_bounds(_, value: (f64 > echo > f64)) -> (() > void > ()),
    "change-value", connect_change_value(_, scroll: (ScrollType > echo > ScrollType),
                                             value: (f64 > echo > f64)) -> (Inhibit > to_glib > gboolean),
    "move-slider", connect_move_slider(_, scroll: (ScrollType > echo > ScrollType)) -> (() > void > ()),
    "value-changed", connect_value_changed(_) -> (() > void > ())
);

impl TreeSelection {
    pub fn connect_changed<F: Fn(TreeSelection) + 'static>(&self, f: F) -> u64 {
        unsafe {
            let f: Box<Box<Fn(TreeSelection) + 'static>> = Box::new(Box::new(f));
            connect(self.unwrap_gobject() as *mut _, "changed",
                transmute(tree_selection_trampoline), into_raw(f) as *mut _)
        }
    }
}

extern "C" fn tree_selection_trampoline(this: *mut GtkTreeSelection,
        f: &Box<Fn(TreeSelection) + 'static>) {
    f(TreeSelection::wrap_object(this as *mut _))
}

signal_trait!(AdjustmentSignals, Adjustment,
    connect_value_changed(_)
);
signal_impl!(
    gobject, AdjustmentSignals, Adjustment, GtkAdjustment,
    "value-changed", connect_value_changed(_) -> (() > void > ())
);


signal_trait!(TreeViewColumnSignals, TreeViewColumn,
    connect_clicked(_)
);
signal_impl!(gobject, TreeViewColumnSignals, TreeViewColumn, GtkTreeViewColumn,
    "clicked", connect_clicked(_) -> (() > void > ())
);


signal_trait!(ExpanderSignals, ::Expander,
    connect_activate(_)
);
signal_impl!(widget, ExpanderSignals, ::Expander, ::ffi::GtkExpander,
    "activate", connect_activate(_) -> (() > void > ())
);