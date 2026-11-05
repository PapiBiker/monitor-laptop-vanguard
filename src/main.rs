#![windows_subsystem = "windows"]

use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

type Bool = i32;
type Dword = u32;
type Handle = *mut c_void;
type Hwnd = Handle;
type Hinstance = Handle;
type Hdc = Handle;
type Hbrush = Handle;
type Hfont = Handle;
type Hpen = Handle;
type Hcursor = Handle;
type Lparam = isize;
type Lresult = isize;
type Wparam = usize;

const WM_CREATE: u32 = 0x0001;
const WM_DESTROY: u32 = 0x0002;
const WM_PAINT: u32 = 0x000F;
const WM_CLOSE: u32 = 0x0010;
const WM_LBUTTONUP: u32 = 0x0202;
const WM_TIMER: u32 = 0x0113;
const WM_NCCREATE: u32 = 0x0081;
const WM_ERASEBKGND: u32 = 0x0014;
const WM_SETCURSOR: u32 = 0x0020;
const WM_MOUSEMOVE: u32 = 0x0200;
const WM_MOUSELEAVE: u32 = 0x02A3;
const WM_MEMORY_TRIMMED: u32 = 0x8001;
const TME_LEAVE: u32 = 0x0002;
const IDC_ARROW: usize = 32512;
const IDC_HAND: usize = 32649;
const GWLP_USERDATA: i32 = -21;
const GWL_STYLE: i32 = -16;
const WS_CHILD: u32 = 0x4000_0000;
const WS_POPUP: u32 = 0x8000_0000;
const WS_EX_TOOLWINDOW: u32 = 0x0000_0080;
const WS_EX_LAYERED: u32 = 0x0008_0000;
const LWA_ALPHA: u32 = 0x0000_0002;
const SW_SHOWNOACTIVATE: i32 = 4;
const SW_SHOWNORMAL: i32 = 1;
const PROCESS_SET_QUOTA: Dword = 0x0100;
const PROCESS_QUERY_LIMITED_INFORMATION: Dword = 0x1000;
const SWP_NOZORDER: u32 = 0x0004;
const SWP_NOACTIVATE: u32 = 0x0010;
const SWP_FRAMECHANGED: u32 = 0x0020;
const SWP_SHOWWINDOW: u32 = 0x0040;
const SM_XVIRTUALSCREEN: i32 = 76;
const SM_YVIRTUALSCREEN: i32 = 77;
const SPI_GETWORKAREA: u32 = 0x0030;
const TRANSPARENT: i32 = 1;
const SRCCOPY: u32 = 0x00CC_0020;
const DC_BRUSH: i32 = 18;
const DC_PEN: i32 = 19;
const DT_LEFT: u32 = 0;
const DT_CENTER: u32 = 1;
const DT_RIGHT: u32 = 2;
const DT_VCENTER: u32 = 4;
const DT_SINGLELINE: u32 = 0x20;
const DT_END_ELLIPSIS: u32 = 0x8000;
const REG_SZ: Dword = 1;
const KEY_SET_VALUE: Dword = 0x0002;
const HKEY_CURRENT_USER: Handle = 0x8000_0001usize as Handle;
const ERROR_SUCCESS: i32 = 0;
const ERROR_ALREADY_EXISTS: u32 = 183;
const DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2: Handle = -4isize as Handle;
const IF_TYPE_SOFTWARE_LOOPBACK: u32 = 24;
const IF_OPER_STATUS_UP: u32 = 1;
const IF_FLAG_HARDWARE: u8 = 0x01;
const IF_FLAG_FILTER: u8 = 0x02;
const MAX_POINTS: usize = 60;
const TIMER_ID: usize = 1;
const DISK_EVERY_TICKS: u32 = 15;
const PLACE_EVERY_TICKS: u32 = 30;
const NOTICE_SECONDS: u64 = 6;

// Medidas logicas a 96 DPI; se escalan segun la pantalla.
const PANEL_WIDTH: i32 = 264;
const PANEL_HEIGHT: i32 = 262;
const PANEL_MARGIN: i32 = 20;
const PANEL_RADIUS: i32 = 14;
const PAD: i32 = 20;
const INNER_RIGHT: i32 = PANEL_WIDTH - PAD;
const BUTTON_SIZE: i32 = 24;
const BUTTON_GAP: i32 = 2;
const BUTTON_TOP: i32 = 14;

// Glifos de Segoe MDL2 Assets (incluida en Windows 10 y 11).
#[derive(Clone, Copy, PartialEq)]
enum Button {
    FreeMemory,
    TaskManager,
    StartupApps,
    Close,
}

const BUTTONS: [(Button, char); 4] = [
    (Button::FreeMemory, '\u{EA99}'),
    (Button::TaskManager, '\u{E9D9}'),
    (Button::StartupApps, '\u{E7E8}'),
    (Button::Close, '\u{E8BB}'),
];

// Los botones se alinean a la derecha del encabezado.
fn button_rect(index: usize) -> Rect {
    let right = INNER_RIGHT + 4 - (BUTTONS.len() - 1 - index) as i32 * (BUTTON_SIZE + BUTTON_GAP);
    rect(
        right - BUTTON_SIZE,
        BUTTON_TOP,
        right,
        BUTTON_TOP + BUTTON_SIZE,
    )
}

// Paleta pensada para el fondo corporativo (gris pizarra #3C4650, logo blanco).
const COLOR_PANEL: u32 = rgb(47, 55, 64);
const COLOR_BORDER: u32 = rgb(74, 84, 95);
const COLOR_TEXT: u32 = rgb(244, 246, 248);
const COLOR_MUTED: u32 = rgb(152, 163, 175);
const COLOR_TRACK: u32 = rgb(66, 76, 87);
const COLOR_AREA: u32 = rgb(68, 80, 93);
const COLOR_LINE: u32 = rgb(226, 232, 238);
const COLOR_ALERT: u32 = rgb(242, 172, 84);

static USER_CLOSED: AtomicBool = AtomicBool::new(false);

const fn rgb(red: u8, green: u8, blue: u8) -> u32 {
    red as u32 | (green as u32) << 8 | (blue as u32) << 16
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct FileTime {
    low: Dword,
    high: Dword,
}

impl FileTime {
    fn value(self) -> u64 {
        ((self.high as u64) << 32) | self.low as u64
    }
}

#[repr(C)]
struct MemoryStatusEx {
    length: Dword,
    memory_load: Dword,
    total_phys: u64,
    avail_phys: u64,
    total_page_file: u64,
    avail_page_file: u64,
    total_virtual: u64,
    avail_virtual: u64,
    avail_extended_virtual: u64,
}

#[repr(C)]
struct PerformanceInformation {
    size: Dword,
    commit_total: usize,
    commit_limit: usize,
    commit_peak: usize,
    physical_total: usize,
    physical_available: usize,
    system_cache: usize,
    kernel_total: usize,
    kernel_paged: usize,
    kernel_nonpaged: usize,
    page_size: usize,
    handle_count: Dword,
    process_count: Dword,
    thread_count: Dword,
}

// MIB_IF_ROW2 de netioapi.h (1352 bytes).
#[repr(C)]
struct MibIfRow2 {
    luid: u64,
    index: u32,
    guid: [u8; 16],
    alias: [u16; 257],
    description: [u16; 257],
    physical_address_length: u32,
    physical_address: [u8; 32],
    permanent_physical_address: [u8; 32],
    mtu: u32,
    if_type: u32,
    tunnel_type: u32,
    media_type: u32,
    physical_medium_type: u32,
    access_type: u32,
    direction_type: u32,
    flags: u8,
    oper_status: u32,
    admin_status: u32,
    media_connect_state: u32,
    network_guid: [u8; 16],
    connection_type: u32,
    transmit_link_speed: u64,
    receive_link_speed: u64,
    in_octets: u64,
    in_other: [u64; 8],
    out_octets: u64,
    out_other: [u64; 8],
}

#[repr(C)]
struct MibIfTable2 {
    num_entries: u32,
    table: [MibIfRow2; 1],
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
struct PaintStruct {
    hdc: Hdc,
    erase: Bool,
    paint: Rect,
    restore: Bool,
    inc_update: Bool,
    rgb_reserved: [u8; 32],
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
struct Msg {
    hwnd: Hwnd,
    message: u32,
    w_param: Wparam,
    l_param: Lparam,
    time: Dword,
    point: Point,
    private: Dword,
}

type WndProc = unsafe extern "system" fn(Hwnd, u32, Wparam, Lparam) -> Lresult;

#[repr(C)]
struct WndClassW {
    style: u32,
    wnd_proc: Option<WndProc>,
    class_extra: i32,
    window_extra: i32,
    instance: Hinstance,
    icon: Handle,
    cursor: Hcursor,
    background: Hbrush,
    menu_name: *const u16,
    class_name: *const u16,
}

#[repr(C)]
struct CreateStructW {
    create_params: *mut c_void,
    instance: Hinstance,
    menu: Handle,
    parent: Hwnd,
    height: i32,
    width: i32,
    y: i32,
    x: i32,
    style: i32,
    name: *const u16,
    class: *const u16,
    ex_style: u32,
}

#[derive(Clone, Copy, Default)]
struct Metrics {
    cpu: f64,
    memory_used: u64,
    memory_total: u64,
    disk_free: u64,
    disk_total: u64,
    network_in: f64,
    network_out: f64,
    process_count: u32,
    uptime_seconds: u64,
}

impl Metrics {
    fn memory_percent(&self) -> f64 {
        percent(self.memory_used, self.memory_total)
    }

    fn disk_used_percent(&self) -> f64 {
        percent(
            self.disk_total.saturating_sub(self.disk_free),
            self.disk_total,
        )
    }
}

// Recursos GDI creados una sola vez y reutilizados en cada cuadro.
struct Gdi {
    buffer_dc: Hdc,
    bitmap: Handle,
    old_bitmap: Handle,
    font_title: Hfont,
    font_label: Hfont,
    font_value: Hfont,
    font_hero: Hfont,
    font_icon: Hfont,
    line_pen: Hpen,
}

struct AppState {
    hovered: Option<usize>,
    tracking_mouse: bool,
    trimming: Option<u64>,
    notice: Option<(String, Instant)>,
    metrics: Metrics,
    previous_idle: u64,
    previous_kernel: u64,
    previous_user: u64,
    previous_net: Option<(u64, u64)>,
    previous_sample: Instant,
    cpu_history: Vec<u8>,
    disk_label: String,
    disk_path: Vec<u16>,
    ticks: u32,
    scale: f32,
    gdi: Option<Gdi>,
}

struct DesktopHostSearch {
    host: Hwnd,
}

impl AppState {
    fn new(scale: f32) -> Self {
        let disk_label = system_drive();
        Self {
            hovered: None,
            tracking_mouse: false,
            trimming: None,
            notice: None,
            metrics: Metrics::default(),
            previous_idle: 0,
            previous_kernel: 0,
            previous_user: 0,
            previous_net: None,
            previous_sample: Instant::now(),
            cpu_history: Vec::with_capacity(MAX_POINTS),
            disk_path: wide_null(&format!("{disk_label}\\")),
            disk_label,
            ticks: 0,
            scale,
            gdi: None,
        }
    }

    fn px(&self, value: i32) -> i32 {
        scaled(value, self.scale)
    }

    fn sample(&mut self) {
        let mut idle = FileTime::default();
        let mut kernel = FileTime::default();
        let mut user = FileTime::default();
        let now = Instant::now();
        let elapsed = now
            .duration_since(self.previous_sample)
            .as_secs_f64()
            .max(0.001);
        self.previous_sample = now;

        unsafe {
            if GetSystemTimes(&mut idle, &mut kernel, &mut user) != 0 {
                let idle_now = idle.value();
                let kernel_now = kernel.value();
                let user_now = user.value();
                let total_delta = kernel_now
                    .saturating_sub(self.previous_kernel)
                    .saturating_add(user_now.saturating_sub(self.previous_user));
                let idle_delta = idle_now.saturating_sub(self.previous_idle);
                if self.previous_kernel != 0 && total_delta > 0 {
                    self.metrics.cpu = (100.0 * (total_delta.saturating_sub(idle_delta)) as f64
                        / total_delta as f64)
                        .clamp(0.0, 100.0);
                }
                self.previous_idle = idle_now;
                self.previous_kernel = kernel_now;
                self.previous_user = user_now;
            }

            let mut memory: MemoryStatusEx = zeroed();
            memory.length = size_of::<MemoryStatusEx>() as Dword;
            if GlobalMemoryStatusEx(&mut memory) != 0 {
                self.metrics.memory_total = memory.total_phys;
                self.metrics.memory_used = memory.total_phys.saturating_sub(memory.avail_phys);
            }

            // El disco cambia lento: no hace falta consultarlo cada segundo.
            if self.ticks.is_multiple_of(DISK_EVERY_TICKS) {
                let mut free = 0u64;
                let mut total = 0u64;
                if GetDiskFreeSpaceExW(self.disk_path.as_ptr(), &mut free, &mut total, null_mut())
                    != 0
                {
                    self.metrics.disk_free = free;
                    self.metrics.disk_total = total;
                }
            }

            if let Some((net_in, net_out)) = network_counters() {
                if let Some((previous_in, previous_out)) = self.previous_net {
                    self.metrics.network_in = net_in.saturating_sub(previous_in) as f64 / elapsed;
                    self.metrics.network_out =
                        net_out.saturating_sub(previous_out) as f64 / elapsed;
                }
                self.previous_net = Some((net_in, net_out));
            }

            let mut performance: PerformanceInformation = zeroed();
            performance.size = size_of::<PerformanceInformation>() as Dword;
            if K32GetPerformanceInfo(&mut performance, performance.size) != 0 {
                self.metrics.process_count = performance.process_count;
            }
            self.metrics.uptime_seconds = GetTickCount64() / 1000;
        }

        push_history(&mut self.cpu_history, self.metrics.cpu);
        self.ticks = self.ticks.wrapping_add(1);
    }
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(name: *const u16) -> Hinstance;
    fn GetSystemTimes(idle: *mut FileTime, kernel: *mut FileTime, user: *mut FileTime) -> Bool;
    fn GlobalMemoryStatusEx(status: *mut MemoryStatusEx) -> Bool;
    fn K32GetPerformanceInfo(info: *mut PerformanceInformation, size: Dword) -> Bool;
    fn GetDiskFreeSpaceExW(
        directory: *const u16,
        free_available: *mut u64,
        total_bytes: *mut u64,
        total_free: *mut u64,
    ) -> Bool;
    fn GetTickCount64() -> u64;
    fn CreateMutexW(attributes: *mut c_void, owner: Bool, name: *const u16) -> Handle;
    fn GetLastError() -> u32;
    fn SetProcessWorkingSetSize(process: Handle, minimum: usize, maximum: usize) -> Bool;
    fn GetCurrentProcess() -> Handle;
    fn K32EnumProcesses(ids: *mut Dword, size: Dword, needed: *mut Dword) -> Bool;
    fn OpenProcess(access: Dword, inherit: Bool, process_id: Dword) -> Handle;
    fn K32EmptyWorkingSet(process: Handle) -> Bool;
    fn CloseHandle(handle: Handle) -> Bool;
}

#[repr(C)]
struct TrackMouseEventInfo {
    size: Dword,
    flags: Dword,
    hwnd: Hwnd,
    hover_time: Dword,
}

#[link(name = "shell32")]
unsafe extern "system" {
    fn ShellExecuteW(
        hwnd: Hwnd,
        operation: *const u16,
        file: *const u16,
        parameters: *const u16,
        directory: *const u16,
        show: i32,
    ) -> Hinstance;
}

#[link(name = "user32")]
unsafe extern "system" {
    fn RegisterClassW(class: *const WndClassW) -> u16;
    fn GetShellWindow() -> Hwnd;
    fn FindWindowExW(
        parent: Hwnd,
        after: Hwnd,
        class_name: *const u16,
        window_name: *const u16,
    ) -> Hwnd;
    fn EnumWindows(
        callback: Option<unsafe extern "system" fn(Hwnd, Lparam) -> Bool>,
        param: Lparam,
    ) -> Bool;
    fn PostMessageW(hwnd: Hwnd, message: u32, w_param: Wparam, l_param: Lparam) -> Bool;
    fn TrackMouseEvent(info: *mut TrackMouseEventInfo) -> Bool;
    fn SetCursor(cursor: Hcursor) -> Hcursor;
    fn SetLayeredWindowAttributes(hwnd: Hwnd, color_key: u32, alpha: u8, flags: u32) -> Bool;
    fn SetParent(hwnd: Hwnd, parent: Hwnd) -> Hwnd;
    fn GetParent(hwnd: Hwnd) -> Hwnd;
    fn SetWindowPos(
        hwnd: Hwnd,
        insert_after: Hwnd,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        flags: u32,
    ) -> Bool;
    fn CreateWindowExW(
        ex_style: Dword,
        class_name: *const u16,
        window_name: *const u16,
        style: Dword,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: Hwnd,
        menu: Handle,
        instance: Hinstance,
        param: *mut c_void,
    ) -> Hwnd;
    fn DefWindowProcW(hwnd: Hwnd, message: u32, w_param: Wparam, l_param: Lparam) -> Lresult;
    fn DestroyWindow(hwnd: Hwnd) -> Bool;
    fn IsWindow(hwnd: Hwnd) -> Bool;
    fn ShowWindow(hwnd: Hwnd, command: i32) -> Bool;
    fn UpdateWindow(hwnd: Hwnd) -> Bool;
    fn GetMessageW(message: *mut Msg, hwnd: Hwnd, min: u32, max: u32) -> Bool;
    fn TranslateMessage(message: *const Msg) -> Bool;
    fn DispatchMessageW(message: *const Msg) -> Lresult;
    fn PostQuitMessage(exit_code: i32);
    fn SetTimer(hwnd: Hwnd, id: usize, interval: u32, callback: *mut c_void) -> usize;
    fn KillTimer(hwnd: Hwnd, id: usize) -> Bool;
    fn InvalidateRect(hwnd: Hwnd, rect: *const Rect, erase: Bool) -> Bool;
    fn BeginPaint(hwnd: Hwnd, paint: *mut PaintStruct) -> Hdc;
    fn EndPaint(hwnd: Hwnd, paint: *const PaintStruct) -> Bool;
    fn GetDC(hwnd: Hwnd) -> Hdc;
    fn ReleaseDC(hwnd: Hwnd, dc: Hdc) -> i32;
    fn FillRect(dc: Hdc, rect: *const Rect, brush: Hbrush) -> i32;
    fn SetWindowLongPtrW(hwnd: Hwnd, index: i32, value: isize) -> isize;
    fn GetWindowLongPtrW(hwnd: Hwnd, index: i32) -> isize;
    fn GetSystemMetrics(index: i32) -> i32;
    fn SystemParametersInfoW(action: u32, param: u32, value: *mut c_void, ini: u32) -> Bool;
    fn SetWindowRgn(hwnd: Hwnd, region: Handle, redraw: Bool) -> i32;
    fn LoadCursorW(instance: Hinstance, cursor_name: *const u16) -> Hcursor;
    fn MessageBoxW(hwnd: Hwnd, text: *const u16, caption: *const u16, kind: u32) -> i32;
    fn SetProcessDpiAwarenessContext(context: Handle) -> Bool;
    fn SetProcessDPIAware() -> Bool;
    fn GetDpiForSystem() -> u32;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    fn CreatePen(style: i32, width: i32, color: u32) -> Hpen;
    fn CreateFontW(
        height: i32,
        width: i32,
        escapement: i32,
        orientation: i32,
        weight: i32,
        italic: Dword,
        underline: Dword,
        strike_out: Dword,
        char_set: Dword,
        output_precision: Dword,
        clip_precision: Dword,
        quality: Dword,
        pitch_and_family: Dword,
        face_name: *const u16,
    ) -> Hfont;
    fn CreateCompatibleDC(dc: Hdc) -> Hdc;
    fn CreateCompatibleBitmap(dc: Hdc, width: i32, height: i32) -> Handle;
    fn DeleteDC(dc: Hdc) -> Bool;
    fn BitBlt(
        dest: Hdc,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        source: Hdc,
        source_x: i32,
        source_y: i32,
        operation: u32,
    ) -> Bool;
    fn CreateRoundRectRgn(
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
        width: i32,
        height: i32,
    ) -> Handle;
    fn GetStockObject(index: i32) -> Handle;
    fn SetDCBrushColor(dc: Hdc, color: u32) -> u32;
    fn SetDCPenColor(dc: Hdc, color: u32) -> u32;
    fn SelectObject(dc: Hdc, object: Handle) -> Handle;
    fn DeleteObject(object: Handle) -> Bool;
    fn SetTextColor(dc: Hdc, color: u32) -> u32;
    fn SetBkMode(dc: Hdc, mode: i32) -> i32;
    fn SetTextCharacterExtra(dc: Hdc, extra: i32) -> i32;
    fn RoundRect(
        dc: Hdc,
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
        width: i32,
        height: i32,
    ) -> Bool;
    fn Polygon(dc: Hdc, points: *const Point, count: i32) -> Bool;
    fn Polyline(dc: Hdc, points: *const Point, count: i32) -> Bool;
    fn DrawTextW(dc: Hdc, text: *const u16, count: i32, rect: *mut Rect, format: u32) -> i32;
}

#[link(name = "iphlpapi")]
unsafe extern "system" {
    fn GetIfTable2(table: *mut *mut MibIfTable2) -> u32;
    fn FreeMibTable(memory: *mut c_void);
}

#[link(name = "advapi32")]
unsafe extern "system" {
    fn RegCreateKeyExW(
        key: Handle,
        subkey: *const u16,
        reserved: Dword,
        class: *mut u16,
        options: Dword,
        desired: Dword,
        security: *mut c_void,
        result: *mut Handle,
        disposition: *mut Dword,
    ) -> i32;
    fn RegSetValueExW(
        key: Handle,
        name: *const u16,
        reserved: Dword,
        value_type: Dword,
        data: *const u8,
        data_size: Dword,
    ) -> i32;
    fn RegDeleteValueW(key: Handle, name: *const u16) -> i32;
    fn RegCloseKey(key: Handle) -> i32;
}

fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn scaled(value: i32, scale: f32) -> i32 {
    (value as f32 * scale).round() as i32
}

fn percent(part: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (100.0 * part as f64 / total as f64).clamp(0.0, 100.0)
    }
}

unsafe extern "system" fn find_desktop_host(hwnd: Hwnd, param: Lparam) -> Bool {
    let search = unsafe { &mut *(param as *mut DesktopHostSearch) };
    search.host = unsafe { shell_view(hwnd) };
    if search.host.is_null() {
        1
    } else {
        0
    }
}

unsafe fn shell_view(parent: Hwnd) -> Hwnd {
    let class = wide_null("SHELLDLL_DefView");
    unsafe { FindWindowExW(parent, null_mut(), class.as_ptr(), null()) }
}

// El panel vive dentro de la vista del escritorio (SHELLDLL_DefView): queda
// detras de las ventanas normales, sigue visible con Win+D y recibe clics.
// En Windows 11 24H2+ esa vista es hija de Progman; antes podia estar en un WorkerW.
unsafe fn desktop_host() -> Result<Hwnd, String> {
    let progman = unsafe { GetShellWindow() };
    if progman.is_null() {
        return Err("No se encontro el escritorio de Windows.".to_string());
    }
    let view = unsafe { shell_view(progman) };
    if !view.is_null() {
        return Ok(view);
    }

    let mut search = DesktopHostSearch { host: null_mut() };
    unsafe {
        EnumWindows(
            Some(find_desktop_host),
            (&mut search as *mut DesktopHostSearch) as Lparam,
        );
    }
    if search.host.is_null() {
        return Err("No se encontro la capa del escritorio (SHELLDLL_DefView).".to_string());
    }
    Ok(search.host)
}

fn system_drive() -> String {
    std::env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string())
}

fn push_history(history: &mut Vec<u8>, value: f64) {
    if history.len() == MAX_POINTS {
        history.remove(0);
    }
    history.push(value.round().clamp(0.0, 100.0) as u8);
}

// Suma solo adaptadores fisicos activos: GetIfTable2 tambien lista filtros
// (WFP, QoS) que repiten el mismo trafico y lo duplicarian.
unsafe fn network_counters() -> Option<(u64, u64)> {
    let mut table: *mut MibIfTable2 = null_mut();
    if unsafe { GetIfTable2(&mut table) } != 0 || table.is_null() {
        return None;
    }

    let mut incoming = 0u64;
    let mut outgoing = 0u64;
    unsafe {
        let count = (*table).num_entries as usize;
        let rows = std::ptr::addr_of!((*table).table).cast::<MibIfRow2>();
        for index in 0..count {
            let row = &*rows.add(index);
            let physical = row.flags & IF_FLAG_HARDWARE != 0 && row.flags & IF_FLAG_FILTER == 0;
            if physical
                && row.if_type != IF_TYPE_SOFTWARE_LOOPBACK
                && row.oper_status == IF_OPER_STATUS_UP
            {
                incoming = incoming.wrapping_add(row.in_octets);
                outgoing = outgoing.wrapping_add(row.out_octets);
            }
        }
        FreeMibTable(table.cast());
    }
    Some((incoming, outgoing))
}

// Pide a cada proceso del usuario que devuelva las paginas que no esta usando.
// Windows las pasa a la lista en espera; si la app las vuelve a necesitar las
// recupera sin perder datos. Sin permisos de administrador omite los procesos
// del sistema.
fn trim_working_sets() {
    let mut ids = vec![0 as Dword; 4096];
    let mut needed = 0;
    let size = (ids.len() * size_of::<Dword>()) as Dword;
    unsafe {
        if K32EnumProcesses(ids.as_mut_ptr(), size, &mut needed) == 0 {
            return;
        }
        let count = needed as usize / size_of::<Dword>();
        for &id in &ids[..count] {
            let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SET_QUOTA, 0, id);
            if !process.is_null() {
                K32EmptyWorkingSet(process);
                CloseHandle(process);
            }
        }
    }
}

fn open_shell(target: &str) {
    let operation = wide_null("open");
    let target = wide_null(target);
    unsafe {
        ShellExecuteW(
            null_mut(),
            operation.as_ptr(),
            target.as_ptr(),
            null(),
            null(),
            SW_SHOWNORMAL,
        );
    }
}

fn hit_button(state: &AppState, x: i32, y: i32) -> Option<usize> {
    (0..BUTTONS.len()).find(|&index| {
        let area = button_rect(index);
        (state.px(area.left)..state.px(area.right)).contains(&x)
            && (state.px(area.top)..state.px(area.bottom)).contains(&y)
    })
}

unsafe fn press_button(hwnd: Hwnd, state: &mut AppState, button: Button) {
    match button {
        Button::FreeMemory => {
            if state.trimming.is_some() {
                return;
            }
            state.trimming = Some(state.metrics.memory_used);
            let target = hwnd as usize;
            std::thread::spawn(move || {
                trim_working_sets();
                std::thread::sleep(Duration::from_millis(600));
                unsafe {
                    PostMessageW(target as Hwnd, WM_MEMORY_TRIMMED, 0, 0);
                }
            });
        }
        Button::TaskManager => open_shell("taskmgr.exe"),
        Button::StartupApps => open_shell("ms-settings:startupapps"),
        Button::Close => {
            USER_CLOSED.store(true, Ordering::Relaxed);
            unsafe {
                DestroyWindow(hwnd);
            }
        }
    }
}

fn trimmed_notice(before: u64, after: u64) -> String {
    let freed = before.saturating_sub(after);
    if freed >= 64 * 1_048_576 {
        format!("Liberados {}", format_bytes(freed))
    } else {
        "La memoria ya estaba optimizada".to_string()
    }
}

fn format_bytes(value: u64) -> String {
    let gib = value as f64 / 1_073_741_824.0;
    if gib >= 100.0 {
        format!("{gib:.0} GB")
    } else if gib >= 1.0 {
        format!("{gib:.1} GB")
    } else {
        format!("{:.0} MB", value as f64 / 1_048_576.0)
    }
}

fn format_rate(bytes_per_second: f64) -> String {
    let mib = bytes_per_second / 1_048_576.0;
    if mib >= 1.0 {
        format!("{mib:.1} MB/s")
    } else {
        format!("{:.0} KB/s", bytes_per_second / 1024.0)
    }
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds / 3_600) % 24;
    let minutes = (seconds / 60) % 60;
    if days > 0 {
        format!("{days} d {hours:02} h")
    } else {
        format!("{hours} h {minutes:02} min")
    }
}

fn rect(left: i32, top: i32, right: i32, bottom: i32) -> Rect {
    Rect {
        left,
        top,
        right,
        bottom,
    }
}

// Convierte a UTF-16 en la pila para no reservar memoria en cada cuadro.
unsafe fn text(dc: Hdc, font: Hfont, value: &str, bounds: Rect, color: u32, format: u32) {
    let mut buffer = [0u16; 80];
    let mut length = 0;
    for unit in value.encode_utf16().take(buffer.len()) {
        buffer[length] = unit;
        length += 1;
    }
    let mut bounds = bounds;
    unsafe {
        SelectObject(dc, font);
        SetTextColor(dc, color);
        DrawTextW(
            dc,
            buffer.as_ptr(),
            length as i32,
            &mut bounds,
            format | DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS,
        );
    }
}

unsafe fn use_dc_colors(dc: Hdc, fill: u32, outline: u32) {
    unsafe {
        SelectObject(dc, GetStockObject(DC_BRUSH));
        SelectObject(dc, GetStockObject(DC_PEN));
        SetDCBrushColor(dc, fill);
        SetDCPenColor(dc, outline);
    }
}

unsafe fn fill(dc: Hdc, area: Rect, color: u32) {
    unsafe {
        SetDCBrushColor(dc, color);
        FillRect(dc, &area, GetStockObject(DC_BRUSH));
    }
}

unsafe fn bar(dc: Hdc, area: Rect, value: f64, color: u32) {
    let height = area.bottom - area.top;
    unsafe {
        use_dc_colors(dc, COLOR_TRACK, COLOR_TRACK);
        RoundRect(
            dc,
            area.left,
            area.top,
            area.right,
            area.bottom,
            height,
            height,
        );
        let width = ((area.right - area.left) as f64 * value / 100.0).round() as i32;
        if width > 0 {
            use_dc_colors(dc, color, color);
            RoundRect(
                dc,
                area.left,
                area.top,
                area.left + width.max(height),
                area.bottom,
                height,
                height,
            );
        }
    }
}

unsafe fn sparkline(dc: Hdc, area: Rect, history: &[u8], pen: Hpen) {
    unsafe {
        fill(
            dc,
            rect(area.left, area.bottom, area.right, area.bottom + 1),
            COLOR_TRACK,
        );
    }
    let count = history.len();
    if count < 2 {
        return;
    }

    let width = area.right - area.left;
    let height = area.bottom - area.top;
    let offset = MAX_POINTS - count;
    let mut points = [Point::default(); MAX_POINTS + 2];
    for (index, value) in history.iter().enumerate() {
        points[index] = Point {
            x: area.left + ((offset + index) as i32 * width) / (MAX_POINTS as i32 - 1),
            y: area.bottom - (*value as i32 * height) / 100,
        };
    }
    points[count] = Point {
        x: points[count - 1].x,
        y: area.bottom,
    };
    points[count + 1] = Point {
        x: points[0].x,
        y: area.bottom,
    };

    unsafe {
        use_dc_colors(dc, COLOR_AREA, COLOR_AREA);
        Polygon(dc, points.as_ptr(), count as i32 + 2);
        SelectObject(dc, pen);
        Polyline(dc, points.as_ptr(), count as i32);
    }
}

// Marca en V que recuerda a las lineas del logo corporativo.
unsafe fn chevron(dc: Hdc, state: &AppState, pen: Hpen) {
    let px = |value: i32| state.px(value);
    let outer = [
        Point {
            x: px(PAD),
            y: px(21),
        },
        Point {
            x: px(PAD + 7),
            y: px(32),
        },
        Point {
            x: px(PAD + 14),
            y: px(21),
        },
    ];
    let inner = [
        Point {
            x: px(PAD + 4),
            y: px(21),
        },
        Point {
            x: px(PAD + 7),
            y: px(26),
        },
        Point {
            x: px(PAD + 10),
            y: px(21),
        },
    ];
    unsafe {
        SelectObject(dc, pen);
        Polyline(dc, outer.as_ptr(), 3);
        SelectObject(dc, GetStockObject(DC_PEN));
        SetDCPenColor(dc, COLOR_MUTED);
        Polyline(dc, inner.as_ptr(), 3);
    }
}

unsafe fn metric_row(
    dc: Hdc,
    state: &AppState,
    gdi: &Gdi,
    top: i32,
    label: &str,
    value: &str,
    value_color: u32,
) {
    let px = |value: i32| state.px(value);
    unsafe {
        text(
            dc,
            gdi.font_label,
            label,
            rect(px(PAD), px(top), px(INNER_RIGHT), px(top + 16)),
            COLOR_MUTED,
            DT_LEFT,
        );
        text(
            dc,
            gdi.font_value,
            value,
            rect(px(PAD + 70), px(top), px(INNER_RIGHT), px(top + 16)),
            value_color,
            DT_RIGHT,
        );
    }
}

unsafe fn render(state: &AppState, gdi: &Gdi) {
    let dc = gdi.buffer_dc;
    let px = |value: i32| state.px(value);
    let metrics = &state.metrics;
    let width = px(PANEL_WIDTH);
    let height = px(PANEL_HEIGHT);

    unsafe {
        SetBkMode(dc, TRANSPARENT);
        use_dc_colors(dc, COLOR_PANEL, COLOR_BORDER);
        RoundRect(dc, 0, 0, width, height, px(PANEL_RADIUS), px(PANEL_RADIUS));

        chevron(dc, state, gdi.line_pen);
        SetTextCharacterExtra(dc, px(2));
        text(
            dc,
            gdi.font_title,
            "MONITOR",
            rect(px(PAD + 22), px(16), px(button_rect(0).left), px(36)),
            COLOR_TEXT,
            DT_LEFT,
        );
        SetTextCharacterExtra(dc, 0);
        for (index, (button, glyph)) in BUTTONS.iter().enumerate() {
            let area = button_rect(index);
            let area = rect(px(area.left), px(area.top), px(area.right), px(area.bottom));
            let busy = *button == Button::FreeMemory && state.trimming.is_some();
            if state.hovered == Some(index) || busy {
                use_dc_colors(dc, COLOR_TRACK, COLOR_TRACK);
                RoundRect(
                    dc,
                    area.left,
                    area.top,
                    area.right,
                    area.bottom,
                    px(8),
                    px(8),
                );
            }
            let color = if busy {
                COLOR_ALERT
            } else if state.hovered == Some(index) {
                COLOR_TEXT
            } else {
                COLOR_MUTED
            };
            let mut buffer = [0u8; 4];
            text(
                dc,
                gdi.font_icon,
                glyph.encode_utf8(&mut buffer),
                area,
                color,
                DT_CENTER,
            );
        }
        fill(
            dc,
            rect(px(PAD), px(46), px(INNER_RIGHT), px(46) + 1),
            COLOR_TRACK,
        );

        // Procesador: cifra grande y grafico del ultimo minuto.
        let cpu_color = if metrics.cpu >= 85.0 {
            COLOR_ALERT
        } else {
            COLOR_TEXT
        };
        text(
            dc,
            gdi.font_label,
            "PROCESADOR",
            rect(px(PAD), px(56), px(INNER_RIGHT), px(72)),
            COLOR_MUTED,
            DT_LEFT,
        );
        text(
            dc,
            gdi.font_hero,
            &format!("{:.0}%", metrics.cpu),
            rect(px(PAD - 2), px(72), px(PAD + 84), px(108)),
            cpu_color,
            DT_LEFT,
        );
        sparkline(
            dc,
            rect(px(PAD + 88), px(78), px(INNER_RIGHT), px(106)),
            &state.cpu_history,
            gdi.line_pen,
        );

        let memory_percent = metrics.memory_percent();
        let memory_color = if memory_percent >= 90.0 {
            COLOR_ALERT
        } else {
            COLOR_LINE
        };
        metric_row(
            dc,
            state,
            gdi,
            120,
            &format!("MEMORIA  {memory_percent:.0}%"),
            &format!(
                "{} / {}",
                format_bytes(metrics.memory_used),
                format_bytes(metrics.memory_total)
            ),
            COLOR_TEXT,
        );
        bar(
            dc,
            rect(px(PAD), px(142), px(INNER_RIGHT), px(146)),
            memory_percent,
            memory_color,
        );

        let disk_used = metrics.disk_used_percent();
        let disk_color = if metrics.disk_total > 0 && disk_used >= 90.0 {
            COLOR_ALERT
        } else {
            COLOR_LINE
        };
        metric_row(
            dc,
            state,
            gdi,
            156,
            &format!("DISCO {}", state.disk_label),
            &format!("{} libres", format_bytes(metrics.disk_free)),
            COLOR_TEXT,
        );
        bar(
            dc,
            rect(px(PAD), px(178), px(INNER_RIGHT), px(182)),
            disk_used,
            disk_color,
        );

        metric_row(
            dc,
            state,
            gdi,
            194,
            "RED",
            &format!(
                "↓ {}   ↑ {}",
                format_rate(metrics.network_in),
                format_rate(metrics.network_out)
            ),
            COLOR_TEXT,
        );

        fill(
            dc,
            rect(px(PAD), px(222), px(INNER_RIGHT), px(222) + 1),
            COLOR_TRACK,
        );
        // El pie muestra el resultado de un boton durante unos segundos.
        let footer = rect(px(PAD), px(230), px(INNER_RIGHT), px(246));
        let notice = state
            .notice
            .as_ref()
            .filter(|(_, shown)| shown.elapsed().as_secs() < NOTICE_SECONDS);
        if state.trimming.is_some() {
            text(
                dc,
                gdi.font_label,
                "Liberando memoria…",
                footer,
                COLOR_ALERT,
                DT_LEFT,
            );
        } else if let Some((message, _)) = notice {
            text(dc, gdi.font_label, message, footer, COLOR_TEXT, DT_LEFT);
        } else {
            text(
                dc,
                gdi.font_label,
                &format!("{} procesos", metrics.process_count),
                footer,
                COLOR_MUTED,
                DT_LEFT,
            );
            text(
                dc,
                gdi.font_label,
                &format!("encendido {}", format_uptime(metrics.uptime_seconds)),
                rect(px(PAD + 90), px(230), px(INNER_RIGHT), px(246)),
                COLOR_MUTED,
                DT_RIGHT,
            );
        }
    }
}

unsafe fn create_font(size: i32, weight: i32) -> Hfont {
    unsafe { create_font_face(size, weight, "Segoe UI") }
}

unsafe fn create_font_face(size: i32, weight: i32, face: &str) -> Hfont {
    let face = wide_null(face);
    unsafe {
        CreateFontW(
            -size,
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            1,
            0,
            0,
            5,
            0,
            face.as_ptr(),
        )
    }
}

unsafe fn create_gdi(hwnd: Hwnd, state: &AppState) -> Gdi {
    unsafe {
        let window_dc = GetDC(hwnd);
        let buffer_dc = CreateCompatibleDC(window_dc);
        let bitmap =
            CreateCompatibleBitmap(window_dc, state.px(PANEL_WIDTH), state.px(PANEL_HEIGHT));
        ReleaseDC(hwnd, window_dc);
        let old_bitmap = SelectObject(buffer_dc, bitmap);
        Gdi {
            buffer_dc,
            bitmap,
            old_bitmap,
            font_title: create_font(state.px(12), 600),
            font_label: create_font(state.px(12), 400),
            font_value: create_font(state.px(13), 600),
            font_hero: create_font(state.px(34), 300),
            font_icon: create_font_face(state.px(14), 400, "Segoe MDL2 Assets"),
            line_pen: CreatePen(0, state.px(2).max(1), COLOR_LINE),
        }
    }
}

unsafe fn destroy_gdi(gdi: Gdi) {
    unsafe {
        SelectObject(gdi.buffer_dc, gdi.old_bitmap);
        SelectObject(gdi.buffer_dc, GetStockObject(DC_PEN));
        SelectObject(gdi.buffer_dc, GetStockObject(DC_BRUSH));
        DeleteObject(gdi.bitmap);
        DeleteDC(gdi.buffer_dc);
        for object in [
            gdi.font_title,
            gdi.font_label,
            gdi.font_value,
            gdi.font_hero,
            gdi.font_icon,
            gdi.line_pen,
        ] {
            DeleteObject(object);
        }
    }
}

// Esquina superior derecha del area de trabajo, en coordenadas del escritorio.
unsafe fn place_panel(hwnd: Hwnd, state: &AppState) -> bool {
    let mut work = Rect::default();
    unsafe {
        if SystemParametersInfoW(SPI_GETWORKAREA, 0, (&mut work as *mut Rect).cast(), 0) == 0 {
            return false;
        }
        let origin_x = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let origin_y = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let width = state.px(PANEL_WIDTH);
        SetWindowPos(
            hwnd,
            null_mut(),
            work.right - width - state.px(PANEL_MARGIN) - origin_x,
            work.top + state.px(PANEL_MARGIN) - origin_y,
            width,
            state.px(PANEL_HEIGHT),
            SWP_NOZORDER | SWP_NOACTIVATE,
        ) != 0
    }
}

fn mouse_position(l_param: Lparam) -> (i32, i32) {
    (
        (l_param as u16) as i16 as i32,
        ((l_param >> 16) as u16) as i16 as i32,
    )
}

unsafe fn state_of(hwnd: Hwnd) -> *mut AppState {
    unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppState }
}

unsafe extern "system" fn window_proc(
    hwnd: Hwnd,
    message: u32,
    w_param: Wparam,
    l_param: Lparam,
) -> Lresult {
    match message {
        WM_NCCREATE => {
            let create = l_param as *const CreateStructW;
            if !create.is_null() {
                let state = unsafe { (*create).create_params }.cast::<AppState>();
                unsafe {
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize);
                }
            }
            1
        }
        WM_CREATE => {
            let state = unsafe { state_of(hwnd) };
            if !state.is_null() {
                unsafe {
                    (*state).sample();
                    (*state).gdi = Some(create_gdi(hwnd, &*state));
                    SetTimer(hwnd, TIMER_ID, 1000, null_mut());
                }
            }
            0
        }
        WM_TIMER => {
            let state = unsafe { state_of(hwnd) };
            if !state.is_null() {
                unsafe {
                    let state = &mut *state;
                    state.sample();
                    // Si cambia la resolucion o la barra de tareas, se reubica.
                    if state.ticks.is_multiple_of(PLACE_EVERY_TICKS) {
                        place_panel(hwnd, state);
                    }
                    InvalidateRect(hwnd, null(), 0);
                }
            }
            0
        }
        WM_PAINT => {
            let mut paint: PaintStruct = unsafe { zeroed() };
            let state = unsafe { state_of(hwnd) };
            unsafe {
                let dc = BeginPaint(hwnd, &mut paint);
                if let Some(state) = state.as_ref() {
                    if let Some(gdi) = &state.gdi {
                        render(state, gdi);
                        BitBlt(
                            dc,
                            0,
                            0,
                            state.px(PANEL_WIDTH),
                            state.px(PANEL_HEIGHT),
                            gdi.buffer_dc,
                            0,
                            0,
                            SRCCOPY,
                        );
                    }
                }
                EndPaint(hwnd, &paint);
            }
            0
        }
        WM_ERASEBKGND => 1,
        WM_MOUSEMOVE => {
            let state = unsafe { state_of(hwnd) };
            if let Some(state) = unsafe { state.as_mut() } {
                let (x, y) = mouse_position(l_param);
                let hovered = hit_button(state, x, y);
                if !state.tracking_mouse {
                    let mut track = TrackMouseEventInfo {
                        size: size_of::<TrackMouseEventInfo>() as Dword,
                        flags: TME_LEAVE,
                        hwnd,
                        hover_time: 0,
                    };
                    state.tracking_mouse = unsafe { TrackMouseEvent(&mut track) } != 0;
                }
                if hovered != state.hovered {
                    state.hovered = hovered;
                    unsafe {
                        InvalidateRect(hwnd, null(), 0);
                    }
                }
            }
            0
        }
        WM_MOUSELEAVE => {
            let state = unsafe { state_of(hwnd) };
            if let Some(state) = unsafe { state.as_mut() } {
                state.tracking_mouse = false;
                if state.hovered.take().is_some() {
                    unsafe {
                        InvalidateRect(hwnd, null(), 0);
                    }
                }
            }
            0
        }
        WM_SETCURSOR => {
            let state = unsafe { state_of(hwnd) };
            let cursor = match unsafe { state.as_ref() } {
                Some(state) if state.hovered.is_some() => IDC_HAND,
                _ => IDC_ARROW,
            };
            unsafe {
                SetCursor(LoadCursorW(null_mut(), cursor as *const u16));
            }
            1
        }
        WM_LBUTTONUP => {
            let state = unsafe { state_of(hwnd) };
            if let Some(state) = unsafe { state.as_mut() } {
                let (x, y) = mouse_position(l_param);
                if let Some(index) = hit_button(state, x, y) {
                    unsafe {
                        press_button(hwnd, state, BUTTONS[index].0);
                        if IsWindow(hwnd) != 0 {
                            InvalidateRect(hwnd, null(), 0);
                        }
                    }
                }
            }
            0
        }
        WM_MEMORY_TRIMMED => {
            let state = unsafe { state_of(hwnd) };
            if let Some(state) = unsafe { state.as_mut() } {
                if let Some(before) = state.trimming.take() {
                    state.sample();
                    let notice = trimmed_notice(before, state.metrics.memory_used);
                    state.notice = Some((notice, Instant::now()));
                }
                unsafe {
                    InvalidateRect(hwnd, null(), 0);
                }
            }
            0
        }
        WM_CLOSE => {
            USER_CLOSED.store(true, Ordering::Relaxed);
            unsafe {
                DestroyWindow(hwnd);
            }
            0
        }
        WM_DESTROY => {
            let state = unsafe { state_of(hwnd) };
            unsafe {
                KillTimer(hwnd, TIMER_ID);
                if !state.is_null() {
                    let mut state = Box::from_raw(state);
                    if let Some(gdi) = state.gdi.take() {
                        destroy_gdi(gdi);
                    }
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                }
                PostQuitMessage(0);
            }
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, message, w_param, l_param) },
    }
}

const CLASS_NAME: &str = "RustSystemMonitorWindow";

unsafe fn register_class() -> Result<(), String> {
    let class_name = wide_null(CLASS_NAME);
    unsafe {
        let class = WndClassW {
            style: 0,
            wnd_proc: Some(window_proc),
            class_extra: 0,
            window_extra: 0,
            instance: GetModuleHandleW(null()),
            icon: null_mut(),
            cursor: LoadCursorW(null_mut(), IDC_ARROW as *const u16),
            background: null_mut(),
            menu_name: null(),
            class_name: class_name.as_ptr(),
        };
        if RegisterClassW(&class) == 0 {
            return Err("No se pudo registrar la ventana de Windows.".to_string());
        }
    }
    Ok(())
}

fn run_panel(scale: f32) -> Result<(), String> {
    let class_name = wide_null(CLASS_NAME);
    let window_name = wide_null("Monitor del sistema");

    unsafe {
        let host = desktop_host()?;
        let state = Box::into_raw(Box::new(AppState::new(scale)));
        let width = scaled(PANEL_WIDTH, scale);
        let height = scaled(PANEL_HEIGHT, scale);
        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_LAYERED,
            class_name.as_ptr(),
            window_name.as_ptr(),
            WS_POPUP,
            0,
            0,
            width,
            height,
            null_mut(),
            null_mut(),
            GetModuleHandleW(null()),
            state.cast(),
        );
        if hwnd.is_null() {
            drop(Box::from_raw(state));
            return Err("No se pudo crear el panel de monitor.".to_string());
        }

        let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
        SetWindowLongPtrW(hwnd, GWL_STYLE, ((style & !WS_POPUP) | WS_CHILD) as isize);
        SetParent(hwnd, host);
        if GetParent(hwnd) != host {
            DestroyWindow(hwnd);
            return Err("No se pudo colocar el panel en el escritorio de Windows.".to_string());
        }
        // Encima de la lista de iconos para que el panel se vea y reciba clics.
        SetWindowPos(
            hwnd,
            null_mut(),
            0,
            0,
            width,
            height,
            SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_SHOWWINDOW,
        );
        // La composicion del escritorio solo muestra hijos con capa propia.
        SetLayeredWindowAttributes(hwnd, 0, 255, LWA_ALPHA);
        if !place_panel(hwnd, &*state) {
            DestroyWindow(hwnd);
            return Err("No se pudo ubicar el panel en el escritorio.".to_string());
        }
        let radius = scaled(PANEL_RADIUS, scale);
        SetWindowRgn(
            hwnd,
            CreateRoundRectRgn(0, 0, width + 1, height + 1, radius, radius),
            1,
        );
        ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        UpdateWindow(hwnd);
        // Devuelve al sistema las paginas usadas solo durante el arranque.
        SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX);

        let mut message: Msg = zeroed();
        loop {
            let result = GetMessageW(&mut message, null_mut(), 0, 0);
            if result <= 0 {
                if result < 0 {
                    return Err("Error en el bucle de mensajes de Windows.".to_string());
                }
                break;
            }
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
        // Si el Explorador se reinicia, el panel desaparece con el escritorio.
        if IsWindow(hwnd) != 0 {
            DestroyWindow(hwnd);
        }
    }
    Ok(())
}

unsafe fn configure_startup(remove: bool) -> Result<(), String> {
    let subkey = wide_null("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
    let value_name = wide_null("RustSystemMonitor");
    let mut key: Handle = null_mut();
    let result = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            0,
            null_mut(),
            0,
            KEY_SET_VALUE,
            null_mut(),
            &mut key,
            null_mut(),
        )
    };
    if result != ERROR_SUCCESS {
        return Err(format!(
            "No se pudo abrir la configuracion de inicio ({result})."
        ));
    }

    let result = if remove {
        unsafe { RegDeleteValueW(key, value_name.as_ptr()) }
    } else {
        let executable = std::env::current_exe()
            .map_err(|error| format!("No se pudo localizar el ejecutable: {error}"))?;
        let quoted_path = format!("\"{}\"", executable.display());
        let value = wide_null(&quoted_path);
        unsafe {
            RegSetValueExW(
                key,
                value_name.as_ptr(),
                0,
                REG_SZ,
                value.as_ptr().cast(),
                (value.len() * size_of::<u16>()) as Dword,
            )
        }
    };
    unsafe {
        RegCloseKey(key);
    }
    if result != ERROR_SUCCESS {
        return Err(format!(
            "No se pudo actualizar el inicio automatico ({result})."
        ));
    }
    Ok(())
}

fn startup_mode() -> Option<bool> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("--install-startup") => Some(false),
        Some("--remove-startup") => Some(true),
        _ => None,
    }
}

fn show_error(error: &str) {
    let title = wide_null("Monitor Rust");
    let message = wide_null(error);
    unsafe {
        MessageBoxW(null_mut(), message.as_ptr(), title.as_ptr(), 0x10);
    }
}

unsafe fn system_scale() -> f32 {
    unsafe {
        if SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) == 0 {
            SetProcessDPIAware();
        }
        (GetDpiForSystem().max(96) as f32) / 96.0
    }
}

fn main() {
    if let Some(remove) = startup_mode() {
        if let Err(error) = unsafe { configure_startup(remove) } {
            show_error(&error);
        }
        return;
    }

    // Una sola instancia aunque se abra a mano y desde el inicio de Windows.
    let mutex_name = wide_null("Local\\RustSystemMonitor");
    unsafe {
        if CreateMutexW(null_mut(), 1, mutex_name.as_ptr()).is_null()
            || GetLastError() == ERROR_ALREADY_EXISTS
        {
            return;
        }
    }

    let scale = unsafe { system_scale() };
    if let Err(error) = unsafe { register_class() } {
        show_error(&error);
        return;
    }

    let mut first_attempt = true;
    loop {
        let result = run_panel(scale);
        if USER_CLOSED.load(Ordering::Relaxed) {
            break;
        }
        if let Err(error) = result {
            if first_attempt {
                show_error(&error);
                break;
            }
        }
        // El escritorio se cerro (Explorador reiniciado): esperar y volver.
        first_attempt = false;
        std::thread::sleep(Duration::from_secs(3));
    }
}

#[cfg(test)]
mod tests {
    use super::{
        format_bytes, format_rate, format_uptime, percent, push_history, rgb, scaled, MibIfRow2,
        MibIfTable2, PerformanceInformation, MAX_POINTS,
    };
    use std::mem::{offset_of, size_of};

    #[test]
    fn formats_memory_network_and_uptime() {
        assert_eq!(format_bytes(2 * 1_073_741_824), "2.0 GB");
        assert_eq!(format_bytes(348 * 1_073_741_824), "348 GB");
        assert_eq!(format_rate(2.0 * 1_048_576.0), "2.0 MB/s");
        assert_eq!(format_uptime(90_061), "1 d 01 h");
        assert_eq!(format_uptime(68_820), "19 h 07 min");
    }

    #[test]
    fn graph_keeps_only_the_latest_samples() {
        let mut graph = Vec::new();
        for value in 0..=MAX_POINTS {
            push_history(&mut graph, value as f64);
        }
        assert_eq!(graph.len(), MAX_POINTS);
        assert_eq!(graph.first(), Some(&1));
        assert_eq!(graph.last(), Some(&(MAX_POINTS as u8)));
    }

    #[test]
    fn helpers_scale_and_convert() {
        assert_eq!(rgb(0x11, 0x22, 0x33), 0x0033_2211);
        assert_eq!(scaled(20, 1.5), 30);
        assert_eq!(percent(1, 4), 25.0);
        assert_eq!(percent(1, 0), 0.0);
    }

    #[test]
    fn win32_structs_match_the_sdk_layout() {
        assert_eq!(size_of::<MibIfRow2>(), 1352);
        assert_eq!(offset_of!(MibIfRow2, flags), 1152);
        assert_eq!(offset_of!(MibIfRow2, oper_status), 1156);
        assert_eq!(offset_of!(MibIfRow2, in_octets), 1208);
        assert_eq!(offset_of!(MibIfRow2, out_octets), 1280);
        assert_eq!(offset_of!(MibIfTable2, table), 8);
        assert_eq!(size_of::<PerformanceInformation>(), 104);
    }
}

#[cfg(test)]
mod button_tests {
    use super::{button_rect, trimmed_notice, BUTTONS, INNER_RIGHT, PAD};

    #[test]
    fn buttons_fit_in_the_header_without_overlapping() {
        for index in 1..BUTTONS.len() {
            assert!(button_rect(index - 1).right <= button_rect(index).left);
        }
        assert!(button_rect(0).left > PAD + 100);
        assert!(button_rect(BUTTONS.len() - 1).right <= INNER_RIGHT + 4);
    }

    #[test]
    fn notice_reports_freed_memory() {
        assert_eq!(trimmed_notice(3 << 30, 2 << 30), "Liberados 1.0 GB");
        assert_eq!(
            trimmed_notice(1 << 30, 1 << 30),
            "La memoria ya estaba optimizada"
        );
    }
}
