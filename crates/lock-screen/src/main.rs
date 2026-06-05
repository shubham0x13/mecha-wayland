#![recursion_limit = "4096"]

mod atlas {
    include!(concat!(env!("OUT_DIR"), "/ui_gen.rs"));
}

mod time;
mod widgets;

use std::collections::HashMap;
use std::os::fd::AsRawFd;
use utils::Rect;

use ::renderer::commands::{Color, Point};
use app::prelude::State;
use timer::{Timer, TimerEvent, TimerId};
use wayland::Wayland;
use widgets::clock;

const DRM_FORMAT_ARGB8888: u32 = 0x34325241;
const DRAG_TO_UNLOCK_THRESHOLD: f32 = 150.0;

#[derive(Default, Clone, Copy, Debug)]
pub struct HitBoxes {
    pub action_btn: Rect,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum AppStateMode {
    #[default]
    Unlocked,
    Locking,
    Locked,
}

#[derive(Clone, Copy, Debug)]
pub struct OutputInfo {
    pub global_name: u32,
    pub id: u32,
}

pub struct SurfaceState {
    pub wl_surface_id: u32,
    pub lock_surface_id: Option<u32>,
    pub layer_surface_id: Option<u32>,
    pub size: (i32, i32),
    pub dmabuf: [Option<::renderer::RenderableSurface<::renderer::DmaBuf>>; 2],
    pub wl_buf_ids: [u32; 2],
    pub buf_in_flight: [bool; 2],
    pub configured: bool,
    pub hit_boxes: HitBoxes,
    pub frame_callback_pending: bool,
    pub dirty: bool,
}

pub struct UiState {
    pub outputs: Vec<OutputInfo>,
    pub mode: AppStateMode,
    pub lock_id: Option<u32>,
    pub layer_surface: Option<SurfaceState>,
    pub lock_surfaces: HashMap<u32, SurfaceState>,
    pub callback_to_surface: HashMap<u32, u32>,
    pub icon_tex: ::renderer::TextureId,
    pub cursor_x: f64,
    pub cursor_y: f64,
    pub focused_surface_id: Option<u32>,
    pub is_dragging: bool,
    pub drag_start_y: f64,
    pub drag_y_offset: f32,
    pub drag_surface_id: Option<u32>,
}

impl UiState {
    pub fn new(icon_tex: ::renderer::TextureId) -> Self {
        Self {
            outputs: Vec::new(),
            mode: AppStateMode::Unlocked,
            lock_id: None,
            layer_surface: None,
            lock_surfaces: HashMap::new(),
            callback_to_surface: HashMap::new(),
            icon_tex,
            cursor_x: 0.0,
            cursor_y: 0.0,
            focused_surface_id: None,
            is_dragging: false,
            drag_start_y: 0.0,
            drag_y_offset: 0.0,
            drag_surface_id: None,
        }
    }
}

#[derive(State)]
pub struct LockScreenState {
    pub ring: io_ring::Ring,
    pub wayland: Wayland,
    pub renderer: ::renderer::Renderer,
    pub timer: Timer,
    pub clock: clock::ClockWidget,
    pub clock_timer_id: Option<TimerId>,
    pub ui: UiState,
}

impl LockScreenState {
    pub fn new() -> Self {
        let ring = io_ring::Ring::default();
        let wayland = Wayland::new(ring.get_proxy()).expect("failed to create wayland connection");
        let mut renderer = ::renderer::Renderer::new().expect("failed to create renderer");
        let timer = Timer::new(ring.get_proxy());

        use ::renderer::commands::*;
        renderer.init_command_queue::<ClearColor>();
        renderer.init_command_queue::<DrawRect>();
        renderer.init_command_queue::<DrawQuad>();
        renderer.init_command_queue::<DrawMonochromeSprite>();
        renderer.init_command_queue::<DrawText>();

        let icon_tex = renderer
            .upload_atlas(atlas::UI.png_bytes)
            .expect("failed to upload atlas");

        Self {
            ring,
            wayland,
            renderer,
            timer,
            clock: clock::ClockWidget::new(),
            clock_timer_id: None,
            ui: UiState::new(icon_tex),
        }
    }
}

fn alloc_surface_buffers(
    renderer: &mut ::renderer::Renderer,
    wayland: &mut Wayland,
    w: i32,
    h: i32,
) -> (
    [Option<::renderer::RenderableSurface<::renderer::DmaBuf>>; 2],
    [u32; 2],
) {
    let s0 = renderer
        .create_surface::<::renderer::DmaBuf>(w as u32, h as u32)
        .unwrap();
    let s1 = renderer
        .create_surface::<::renderer::DmaBuf>(w as u32, h as u32)
        .unwrap();
    let id0 = create_wl_buffer(wayland, &s0, w, h);
    let id1 = create_wl_buffer(wayland, &s1, w, h);
    wayland.wl_buffer.register(id0);
    wayland.wl_buffer.register(id1);
    ([Some(s0), Some(s1)], [id0, id1])
}

fn begin_drag(ui: &mut UiState, surface_id: u32, start_y: f64) {
    ui.is_dragging = true;
    ui.drag_start_y = start_y;
    ui.drag_y_offset = 0.0;
    ui.drag_surface_id = Some(surface_id);
}

fn cancel_drag(ui: &mut UiState) {
    ui.is_dragging = false;
    ui.drag_y_offset = 0.0;
    ui.drag_surface_id = None;
}

fn main() {
    let state = LockScreenState::new();

    let mut app = app::App::new(state)
        .mount(io_ring::module())
        .mount(wayland::module())
        .mount(timer::module())
        .mount(clock::module())
        .mount(
            app::Module::new()
                .on(|s: &mut LockScreenState, ev: &TimerEvent| {
                    let (h, m, sec, day, mon) = time::try_clock_tick(s.clock_timer_id, ev)?;
                    time::arm_clock(&mut s.timer, &mut s.clock_timer_id, s.clock.precision());
                    Some(clock::ClockUpdate(h, m, sec, day, mon))
                })
                .on(|s: &mut LockScreenState, _: &clock::ClockChanged| {
                    try_redraw_lock_surfaces(s);
                })
                .on(|s: &mut LockScreenState, _: &wayland::Initilised| {
                    use wayland::zwlr_layer_shell::{KeyboardInteractivity, Layer};

                    let outputs = s.wayland.registry.find_all("wl_output");
                    for (name, ver) in outputs {
                        let output_id = s.wayland.alloc_id();
                        s.wayland.registry.bind(name, "wl_output", ver.min(4), output_id);
                        s.ui.outputs.push(OutputInfo { global_name: name, id: output_id });
                    }

                    let wl_surface_id = s.wayland.compositor.create_surface();
                    s.wayland.surface.register(wl_surface_id);

                    let layer_surface_id =
                        s.wayland.layer_shell.get_layer_surface(
                            wl_surface_id, 0, Layer::Top, "lock-screen",
                        );
                    s.wayland.layer_surface.register(layer_surface_id);
                    s.wayland.layer_surface.set_size(layer_surface_id, 400, 360);
                    s.wayland.layer_surface.set_keyboard_interactivity(
                        layer_surface_id, KeyboardInteractivity::OnDemand,
                    );

                    s.ui.layer_surface = Some(SurfaceState {
                        wl_surface_id,
                        lock_surface_id: None,
                        layer_surface_id: Some(layer_surface_id),
                        size: (0, 0),
                        dmabuf: [None, None],
                        wl_buf_ids: [0, 0],
                        buf_in_flight: [false, false],
                        configured: false,
                        hit_boxes: HitBoxes::default(),
                        frame_callback_pending: false,
                        dirty: false,
                    });

                    s.wayland.surface.commit(wl_surface_id);
                    s.wayland.flush();
                    time::arm_clock(&mut s.timer, &mut s.clock_timer_id, s.clock.precision());
                })
                .on(|s: &mut LockScreenState, ev: &wayland::zwlr_layer_shell::LayerSurfaceEvent| {
                    let wayland::zwlr_layer_shell::LayerSurfaceEvent::Configured {
                        id, serial, width, height,
                    } = ev else { return };

                    let Some(ref mut layer_surface) = s.ui.layer_surface else { return };
                    if layer_surface.layer_surface_id != Some(*id) { return; }

                    let w = if *width  == 0 { 400 } else { *width  as i32 };
                    let h = if *height == 0 { 360 } else { *height as i32 };
                    layer_surface.size = (w, h);

                    let (dmabuf, wl_buf_ids) =
                        alloc_surface_buffers(&mut s.renderer, &mut s.wayland, w, h);
                    layer_surface.dmabuf = dmabuf;
                    layer_surface.wl_buf_ids = wl_buf_ids;

                    layer_surface.configured = true;
                    s.wayland.layer_surface.ack_configure(*id, *serial);

                    let success = redraw_surface(
                        &mut s.renderer, &mut s.wayland,
                        &mut s.ui.callback_to_surface, layer_surface,
                        s.ui.icon_tex, &s.clock, false, 0.0,
                    );
                    if success {
                        layer_surface.frame_callback_pending = true;
                        layer_surface.dirty = false;
                    }
                })
                .on(|s: &mut LockScreenState, ev: &wayland::ext_session_lock_surface::ExtSessionLockSurfaceV1Event| {
                    let wayland::ext_session_lock_surface::ExtSessionLockSurfaceV1Event::Configure {
                        id, serial, width, height,
                    } = ev;

                    let found_id = s.ui.lock_surfaces.iter()
                        .find(|(_, ls)| ls.lock_surface_id == Some(*id))
                        .map(|(&wl_id, _)| wl_id);
                    let Some(wl_surface_id) = found_id else { return };

                    let lock_surf = s.ui.lock_surfaces.get_mut(&wl_surface_id).unwrap();
                    let (w, h) = (*width as i32, *height as i32);
                    lock_surf.size = (w, h);

                    if lock_surf.dmabuf[0].is_none() {
                        let (dmabuf, wl_buf_ids) =
                            alloc_surface_buffers(&mut s.renderer, &mut s.wayland, w, h);
                        lock_surf.dmabuf = dmabuf;
                        lock_surf.wl_buf_ids = wl_buf_ids;
                    }

                    lock_surf.configured = true;
                    s.wayland.session_lock_surface.ack_configure(*id, *serial);

                    let success = redraw_surface(
                        &mut s.renderer, &mut s.wayland,
                        &mut s.ui.callback_to_surface, lock_surf,
                        s.ui.icon_tex, &s.clock, false, 0.0,
                    );
                    if success {
                        lock_surf.frame_callback_pending = true;
                        lock_surf.dirty = false;
                    }
                })
                .on(|s: &mut LockScreenState, ev: &wayland::ExtSessionLockEvent| match ev {
                    wayland::ExtSessionLockEvent::Locked => {
                        println!("[client] Session lock activated!");
                        s.ui.mode = AppStateMode::Locked;
                    }
                    wayland::ExtSessionLockEvent::Finished => {
                        println!("[client] Session lock finished/denied!");
                        cleanup_lock_surfaces(s);
                    }
                })
                .on(|s: &mut LockScreenState, ev: &wayland::WlCallbackEvent| {
                    let wayland::WlCallbackEvent::Done { id, .. } = ev;
                    let Some(wl_surface_id) = s.ui.callback_to_surface.remove(id) else { return };
                    let icon_tex = s.ui.icon_tex;

                    let is_dragging    = s.ui.is_dragging;
                    let drag_y_offset  = s.ui.drag_y_offset;
                    let drag_surface_id = s.ui.drag_surface_id;

                    if let Some(ref mut layer_surface) = s.ui.layer_surface {
                        if layer_surface.wl_surface_id == wl_surface_id && layer_surface.configured {
                            layer_surface.frame_callback_pending = false;
                            if layer_surface.dirty {
                                let success = redraw_surface(
                                    &mut s.renderer, &mut s.wayland,
                                    &mut s.ui.callback_to_surface, layer_surface,
                                    icon_tex, &s.clock, false, 0.0,
                                );
                                if success {
                                    layer_surface.frame_callback_pending = true;
                                    layer_surface.dirty = false;
                                }
                            }
                            return;
                        }
                    }

                    if let Some(lock_surf) = s.ui.lock_surfaces.get_mut(&wl_surface_id) {
                        if lock_surf.configured {
                            lock_surf.frame_callback_pending = false;
                            if lock_surf.dirty {
                                let is_active = is_dragging && drag_surface_id == Some(wl_surface_id);
                                let offset = if is_active { drag_y_offset } else { 0.0 };
                                let success = redraw_surface(
                                    &mut s.renderer, &mut s.wayland,
                                    &mut s.ui.callback_to_surface, lock_surf,
                                    icon_tex, &s.clock, is_active, offset,
                                );
                                if success {
                                    lock_surf.frame_callback_pending = true;
                                    lock_surf.dirty = false;
                                }
                            }
                        }
                    }
                })
                .on(|s: &mut LockScreenState, ev: &wayland::WlBufferEvent| {
                    let wayland::WlBufferEvent::Release { id } = ev;
                    let icon_tex = s.ui.icon_tex;

                    let is_dragging = s.ui.is_dragging;
                    let drag_y_offset = s.ui.drag_y_offset;
                    let drag_surface_id = s.ui.drag_surface_id;

                    if let Some(ref mut layer_surface) = s.ui.layer_surface {
                        for i in 0..2 {
                            if layer_surface.wl_buf_ids[i] == *id {
                                layer_surface.buf_in_flight[i] = false;
                                if layer_surface.dirty {
                                    request_redraw(
                                        &mut s.renderer,
                                        &mut s.wayland,
                                        &mut s.ui.callback_to_surface,
                                        layer_surface,
                                        icon_tex,
                                        &s.clock,
                                        false,
                                        0.0,
                                    );
                                }
                                return;
                            }
                        }
                    }

                    let mut found_surf_id = None;
                    for (&wl_id, surf) in &mut s.ui.lock_surfaces {
                        for i in 0..2 {
                            if surf.wl_buf_ids[i] == *id {
                                surf.buf_in_flight[i] = false;
                                if surf.dirty {
                                    found_surf_id = Some(wl_id);
                                }
                                break;
                            }
                        }
                        if found_surf_id.is_some() {
                            break;
                        }
                    }

                    if let Some(wl_id) = found_surf_id {
                        let is_active = is_dragging && drag_surface_id == Some(wl_id);
                        let offset = if is_active { drag_y_offset } else { 0.0 };
                        if let Some(surf) = s.ui.lock_surfaces.get_mut(&wl_id) {
                            request_redraw(
                                &mut s.renderer,
                                &mut s.wayland,
                                &mut s.ui.callback_to_surface,
                                surf,
                                icon_tex,
                                &s.clock,
                                is_active,
                                offset,
                            );
                        }
                    }
                })
                .on(|_: &mut LockScreenState, ev: &wayland::KeyboardEvent| {
                    if let wayland::KeyboardEvent::Key { key, state, .. } = ev {
                        if (*key == 1 || *key == 16) && *state == wayland::KeyState::Pressed {
                            std::process::exit(0);
                        }
                    }
                })
                .on(|s: &mut LockScreenState, ev: &wayland::PointerEvent| match ev {
                    wayland::PointerEvent::Enter { surface, .. } => {
                        s.ui.focused_surface_id = Some(*surface);
                    }
                    wayland::PointerEvent::Leave { surface, .. } => {
                        if s.ui.focused_surface_id == Some(*surface) {
                            s.ui.focused_surface_id = None;
                        }
                        if s.ui.is_dragging {
                            cancel_drag(&mut s.ui);
                            try_redraw_lock_surfaces(s);
                        }
                    }
                    wayland::PointerEvent::Motion { surface_x, surface_y, .. } => {
                        s.ui.cursor_x = *surface_x;
                        s.ui.cursor_y = *surface_y;
                        handle_drag_motion(s, s.ui.cursor_y);
                    }
                    wayland::PointerEvent::Button { state, .. } => {
                        if *state == wayland::ButtonState::Pressed {
                            if let Some(focused_id) = s.ui.focused_surface_id {
                                if let Some(ref layer_surface) = s.ui.layer_surface {
                                    if layer_surface.wl_surface_id == focused_id
                                        && layer_surface.hit_boxes.action_btn
                                            .contains(s.ui.cursor_x, s.ui.cursor_y)
                                    {
                                        trigger_lock(s);
                                        return;
                                    }
                                }
                                if let Some(lock_surf) = s.ui.lock_surfaces.get(&focused_id) {
                                    if lock_surf.hit_boxes.action_btn
                                        .contains(s.ui.cursor_x, s.ui.cursor_y)
                                    {
                                        let cursor_y = s.ui.cursor_y;
                                        begin_drag(&mut s.ui, focused_id, cursor_y);
                                    }
                                }
                            }
                        } else if *state == wayland::ButtonState::Released && s.ui.is_dragging {
                            cancel_drag(&mut s.ui);
                            try_redraw_lock_surfaces(s);
                        }
                    }
                    _ => {}
                })
                .on(|s: &mut LockScreenState, ev: &wayland::TouchEvent| match ev {
                    wayland::TouchEvent::Down { surface, x, y, .. } => {
                        if let Some(ref layer_surface) = s.ui.layer_surface {
                            if layer_surface.wl_surface_id == *surface
                                && layer_surface.hit_boxes.action_btn.contains(*x, *y)
                            {
                                trigger_lock(s);
                                return;
                            }
                        }
                        if let Some(lock_surf) = s.ui.lock_surfaces.get(surface) {
                            if lock_surf.hit_boxes.action_btn.contains(*x, *y) {
                                begin_drag(&mut s.ui, *surface, *y);
                            }
                        }
                    }
                    wayland::TouchEvent::Motion { y, .. } => {
                        handle_drag_motion(s, *y);
                    }
                    wayland::TouchEvent::Up { .. } | wayland::TouchEvent::Cancel => {
                        if s.ui.is_dragging {
                            cancel_drag(&mut s.ui);
                            try_redraw_lock_surfaces(s);
                        }
                    }
                    _ => {}
                })
        );

    app.dispatch(&app::Start);
    loop {
        app.dispatch(&app::PrePoll);
        app.dispatch(&app::Poll);
    }
}

/// Shared motion handler for both pointer and touch drag updates.
fn handle_drag_motion(s: &mut LockScreenState, current_y: f64) {
    if !s.ui.is_dragging {
        return;
    }

    let drag_surf_id = match s.ui.drag_surface_id {
        Some(id) => id,
        None => return,
    };
    // For pointer events we also check the focused surface matches; for touch
    // the surface is implicit in the touch sequence so we always proceed.
    if s.ui.focused_surface_id.is_some() && s.ui.focused_surface_id != Some(drag_surf_id) {
        return;
    }

    let delta_y = (current_y - s.ui.drag_start_y) as f32;
    if delta_y <= 0.0 {
        s.ui.drag_y_offset = delta_y.max(-DRAG_TO_UNLOCK_THRESHOLD);
        if s.ui.drag_y_offset <= -DRAG_TO_UNLOCK_THRESHOLD {
            cancel_drag(&mut s.ui);
            trigger_unlock(s);
            return;
        }
    } else {
        s.ui.drag_y_offset = 0.0;
    }
    try_redraw_lock_surfaces(s);
}

// ---------------------------------------------------------------------------
// Lock / unlock
// ---------------------------------------------------------------------------

fn trigger_lock(s: &mut LockScreenState) {
    if s.ui.mode != AppStateMode::Unlocked {
        return;
    }

    s.ui.mode = AppStateMode::Locking;
    let lock_id = s.wayland.alloc_id();
    s.wayland.session_lock.set_id(lock_id);
    s.wayland.session_lock_manager.lock(lock_id);
    s.ui.lock_id = Some(lock_id);

    let outputs = s.wayland.registry.find_all("wl_output");
    if outputs.is_empty() {
        return;
    }

    for (name, ver) in outputs {
        let output_id = match s.ui.outputs.iter().find(|o| o.global_name == name) {
            Some(o) => o.id,
            None => {
                let id = s.wayland.alloc_id();
                s.wayland.registry.bind(name, "wl_output", ver.min(4), id);
                s.ui.outputs.push(OutputInfo {
                    global_name: name,
                    id,
                });
                id
            }
        };

        let wl_surface_id = s.wayland.compositor.create_surface();
        s.wayland.surface.register(wl_surface_id);

        let lock_surface_id = s.wayland.alloc_id();
        s.wayland
            .session_lock
            .get_lock_surface(lock_surface_id, wl_surface_id, output_id);
        s.wayland.session_lock_surface.register(lock_surface_id);

        s.ui.lock_surfaces.insert(
            wl_surface_id,
            SurfaceState {
                wl_surface_id,
                lock_surface_id: Some(lock_surface_id),
                layer_surface_id: None,
                size: (0, 0),
                dmabuf: [None, None],
                wl_buf_ids: [0, 0],
                buf_in_flight: [false, false],
                configured: false,
                hit_boxes: HitBoxes::default(),
                frame_callback_pending: false,
                dirty: false,
            },
        );
    }
    s.wayland.flush();
}

fn trigger_unlock(s: &mut LockScreenState) {
    if s.ui.mode != AppStateMode::Locked {
        return;
    }
    s.wayland.session_lock.unlock_and_destroy();
    cleanup_lock_surfaces(s);
}

fn cleanup_lock_surfaces(s: &mut LockScreenState) {
    for (wl_surface_id, lock_surf) in std::mem::take(&mut s.ui.lock_surfaces) {
        if let Some(ls_id) = lock_surf.lock_surface_id {
            s.wayland.session_lock_surface.destroy(ls_id);
        }
        s.wayland.send_destroy_surface(wl_surface_id);
        for buf_id in lock_surf.wl_buf_ids {
            if buf_id != 0 {
                s.wayland.send_destroy_buffer(buf_id);
            }
        }
        for dmabuf in lock_surf.dmabuf.into_iter().flatten() {
            s.renderer.destroy_surface(dmabuf);
        }
    }
    s.ui.lock_id = None;
    s.ui.mode = AppStateMode::Unlocked;

    let sync_cb = s.wayland.alloc_id();
    s.wayland.display.sync(sync_cb);
    s.wayland.flush();

    if let Some(ref mut layer_surface) = s.ui.layer_surface {
        request_redraw(
            &mut s.renderer,
            &mut s.wayland,
            &mut s.ui.callback_to_surface,
            layer_surface,
            s.ui.icon_tex,
            &s.clock,
            false,
            0.0,
        );
    }
}

fn try_redraw_lock_surfaces(s: &mut LockScreenState) {
    let icon_tex = s.ui.icon_tex;

    let is_dragging = s.ui.is_dragging;
    let drag_y_offset = s.ui.drag_y_offset;
    let drag_surface_id = s.ui.drag_surface_id;

    for (wl_surf_id, lock_surf) in &mut s.ui.lock_surfaces {
        if !lock_surf.configured {
            continue;
        }
        let is_active = is_dragging && drag_surface_id == Some(*wl_surf_id);
        let offset = if is_active { drag_y_offset } else { 0.0 };
        request_redraw(
            &mut s.renderer,
            &mut s.wayland,
            &mut s.ui.callback_to_surface,
            lock_surf,
            icon_tex,
            &s.clock,
            is_active,
            offset,
        );
    }
}

fn request_redraw(
    renderer: &mut ::renderer::Renderer,
    wayland: &mut Wayland,
    callback_to_surface: &mut HashMap<u32, u32>,
    surface: &mut SurfaceState,
    icon_tex: ::renderer::TextureId,
    clock: &clock::ClockWidget,
    is_active_drag: bool,
    drag_y_offset: f32,
) {
    if surface.frame_callback_pending {
        surface.dirty = true;
    } else {
        let success = redraw_surface(
            renderer,
            wayland,
            callback_to_surface,
            surface,
            icon_tex,
            clock,
            is_active_drag,
            drag_y_offset,
        );
        if success {
            surface.frame_callback_pending = true;
            surface.dirty = false;
        }
    }
}

fn redraw_surface(
    renderer: &mut ::renderer::Renderer,
    wayland: &mut Wayland,
    callback_to_surface: &mut HashMap<u32, u32>,
    surface: &mut SurfaceState,
    icon_tex: ::renderer::TextureId,
    clock: &clock::ClockWidget,
    is_active_drag: bool,
    drag_y_offset: f32,
) -> bool {
    let t0 = std::time::Instant::now();

    let free_idx = if !surface.buf_in_flight[0] {
        0
    } else if !surface.buf_in_flight[1] {
        1
    } else {
        return false;
    };

    renderer.active_surface(surface.dmabuf[free_idx].as_ref().unwrap());

    let (w, h) = surface.size;
    surface.hit_boxes = if surface.layer_surface_id.is_some() {
        render_layer_ui(renderer, w as f32, h as f32, icon_tex)
    } else {
        render_lock_ui(
            renderer,
            w as f32,
            h as f32,
            icon_tex,
            clock,
            is_active_drag,
            drag_y_offset,
        )
    };
    renderer.finish();

    wayland
        .surface
        .attach(surface.wl_surface_id, surface.wl_buf_ids[free_idx], 0, 0);
    wayland.surface.damage(surface.wl_surface_id, 0, 0, w, h);

    let cb_id = wayland.surface.frame(surface.wl_surface_id);
    wayland.callback.register_frame(cb_id);
    callback_to_surface.insert(cb_id, surface.wl_surface_id);

    wayland.surface.commit(surface.wl_surface_id);
    surface.buf_in_flight[free_idx] = true;
    wayland.flush();

    println!(
        "[redraw] frame time: {}µs (buf_idx={})",
        t0.elapsed().as_micros(),
        free_idx
    );

    true
}

fn draw_centered_text(
    renderer: &mut ::renderer::Renderer,
    font: &'static assets::BakedFont,
    texture_id: ::renderer::TextureId,
    text: &str,
    box_bounds: &Rect,
    z_index: f32,
    color: Color,
) {
    let text_w = font.measure_width(text);
    let center_x = box_bounds.x() + (box_bounds.width() - text_w) / 2.0;
    let center_y = box_bounds.y() + font.get_baseline_offset(box_bounds.height());

    renderer.send_command(::renderer::commands::DrawText {
        font,
        texture_id,
        text: text.to_string(),
        origin: Point::new(center_x, center_y),
        z: z_index,
        color,
    });
}

fn render_layer_ui(
    renderer: &mut ::renderer::Renderer,
    win_w: f32,
    win_h: f32,
    icon_tex: ::renderer::TextureId,
) -> HitBoxes {
    use ::renderer::commands::*;
    use layout::layout;

    let mut hit_boxes = HitBoxes::default();
    renderer.send_command(ClearColor::rgb(0.12, 0.12, 0.14));

    layout!(
        {
            available_width: win_w,
            available_height: win_h,
            direction: column,
            padding_top: (win_h - 56.0) / 2.0,
            padding_left: (win_w - 220.0) / 2.0,

            layout!({ width: 220.0, height: 56.0 }, {
                let btn_bb = Rect::xywh(x, y, width, height);
                renderer.send_command(DrawQuad {
                    color: Color::rgb(0.2, 0.4, 0.9),
                    border_color: Color::rgb(0.3, 0.5, 1.0),
                    origin: Point::new(x, y),
                    z: 0.2,
                    size: Size::new(width, height),
                    border_radius: 12.0,
                    border_thickness: 2.0,
                });
                draw_centered_text(
                    renderer, &atlas::UI_FONT_INTER_24, icon_tex,
                    "Lock Session", &btn_bb, 0.6, Color::rgb(1.0, 1.0, 1.0),
                );
                hit_boxes.action_btn = btn_bb;
            }),
        },
        {}
    );

    renderer.process_command_queue::<ClearColor>();
    renderer.process_command_queue::<DrawQuad>();
    renderer.process_command_queue::<DrawText>();
    renderer.process_command_queue::<DrawMonochromeSprite>();
    hit_boxes
}

fn render_lock_ui(
    renderer: &mut ::renderer::Renderer,
    win_w: f32,
    win_h: f32,
    icon_tex: ::renderer::TextureId,
    clock: &clock::ClockWidget,
    is_active_drag: bool,
    drag_y_offset: f32,
) -> HitBoxes {
    use ::renderer::commands::*;
    use layout::layout;

    let mut hit_boxes = HitBoxes::default();
    renderer.send_command(ClearColor::rgb(0.05, 0.05, 0.07));

    let radius = 36.0;
    let btn_w = radius * 2.0;
    let btn_h = radius * 2.0;

    layout!(
        {
            available_width: win_w,
            available_height: win_h,
            direction: column,
            padding_top: 110.0,

            // Clock
            layout!({ height: 80.0 }, {
                let bb = Rect::xywh(x, y, width, height);
                draw_centered_text(
                    renderer, &atlas::UI_FONT_MONO_100, icon_tex,
                    &clock.time_str, &bb, 0.6, Color::rgb(1.0, 1.0, 1.0),
                );
            }),

            // Spacer
            layout!({ height: win_h - 265.0 - btn_h }, {}),

            // Draggable circle button
            layout!({ height: btn_h }, {
                let btn_x = x + (width - btn_w) / 2.0;
                let btn_y = if is_active_drag { y + drag_y_offset } else { y };

                renderer.send_command(DrawQuad {
                    color: Color::from_rgb8(255, 255, 255),
                    border_color: Color::from_rgb8(89, 89, 89),
                    origin: Point::new(btn_x, btn_y),
                    z: 0.2,
                    size: Size::new(btn_w, btn_h),
                    border_radius: radius,
                    border_thickness: 12.0,
                });
                hit_boxes.action_btn = Rect::xywh(btn_x, y, btn_w, btn_h);
            }),

            // Spacer
            layout!({ height: 20.0 }, {}),

            // Help text
            layout!({ height: 25.0 }, {
                let bb = Rect::xywh(x, y, width, height);
                draw_centered_text(
                    renderer, &atlas::UI_FONT_INTER_16, icon_tex,
                    "Swipe up to unlock", &bb, 0.5, Color::rgb(0.6, 0.6, 0.6),
                );
            }),
        },
        {}
    );

    renderer.process_command_queue::<ClearColor>();
    renderer.process_command_queue::<DrawQuad>();
    renderer.process_command_queue::<DrawText>();
    renderer.process_command_queue::<DrawMonochromeSprite>();
    hit_boxes
}

fn create_wl_buffer(
    wayland: &mut Wayland,
    surface: &::renderer::RenderableSurface<::renderer::DmaBuf>,
    width: i32,
    height: i32,
) -> u32 {
    let modifier = surface.backend.modifier;
    let modifier_hi = (modifier >> 32) as u32;
    let modifier_lo = (modifier & 0xffff_ffff) as u32;

    let fd = unsafe { libc::dup(surface.backend.prime_fd.as_raw_fd()) };
    if fd < 0 {
        panic!(
            "failed to dup prime fd: {}",
            std::io::Error::last_os_error()
        );
    }

    let params_id = wayland.dmabuf.create_params();
    wayland.buf_params.register(params_id);
    wayland.buf_params.add(
        params_id,
        fd,
        0,
        0,
        surface.backend.stride,
        modifier_hi,
        modifier_lo,
    );
    let buf_id = wayland
        .buf_params
        .create_immed(params_id, width, height, DRM_FORMAT_ARGB8888, 0);
    wayland.buf_params.destroy(params_id);
    buf_id
}
