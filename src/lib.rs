use std::ffi;
use std::ptr;
use std::rc::Rc;
use std::cell::RefCell;
use std::slice;

extern crate libc;
use libc::{
    c_char,
    c_void,
    c_int,
    uint32_t,
    int32_t,
};

#[macro_use]
extern crate bitflags;

extern crate libfreenect_sys;
use libfreenect_sys::{
    freenect_context,
    freenect_init,
    freenect_shutdown,
    freenect_loglevel,
    freenect_video_format,
    freenect_depth_format,
    freenect_resolution,
    freenect_frame_mode,
    freenect_set_log_level,
    freenect_process_events,
    freenect_num_devices,
    freenect_device_flags,
    freenect_device_attributes,
    freenect_list_device_attributes,
    freenect_free_device_attributes,
    freenect_supported_subdevices,
    freenect_select_subdevices,
    freenect_enabled_subdevices,
    freenect_device,
    freenect_open_device,
    freenect_open_device_by_camera_serial,
    freenect_close_device,
    freenect_set_depth_callback,
    freenect_set_video_callback,
    freenect_get_user,
    freenect_set_user,
    freenect_start_depth,
    freenect_start_video,
    freenect_stop_depth,
    freenect_stop_video,
    freenect_get_video_mode_count,
    freenect_get_video_mode,
    freenect_get_current_video_mode,
    freenect_find_video_mode,
    freenect_set_video_mode,
    freenect_get_depth_mode_count,
	freenect_get_depth_mode,
	freenect_get_current_depth_mode,
	freenect_find_depth_mode,
	freenect_set_depth_mode,
};

#[derive(Debug)]
enum FreenectError {
    LibraryReturnCode(i32),
    NullPtr,
    FrameFormatMismatch,
}

// Error type for the library
pub type FreenectResult<T> = Result<T, FreenectError>;

#[derive(Debug)]
pub enum LogLevel {
	Fatal,         // Log for crashing/non-recoverable errors
	Error,         // Log for major errors
	Warning,       // Log for warning messages
	Notice,        // Log for important messages
	Info,          // Log for normal messages
	Debug,         // Log for useful development messages
	Spew,          // Log for slightly less useful messages
	Flood,         // Log EVERYTHING. May slow performance.
}

impl LogLevel {
    fn to_lowlevel(&self) -> freenect_loglevel {
        match *self {
            LogLevel::Fatal     => freenect_loglevel::FREENECT_LOG_FATAL,
            LogLevel::Error     => freenect_loglevel::FREENECT_LOG_ERROR,
            LogLevel::Warning   => freenect_loglevel::FREENECT_LOG_WARNING,
            LogLevel::Notice    => freenect_loglevel::FREENECT_LOG_NOTICE,
            LogLevel::Info      => freenect_loglevel::FREENECT_LOG_INFO,
            LogLevel::Debug     => freenect_loglevel::FREENECT_LOG_DEBUG,
            LogLevel::Spew      => freenect_loglevel::FREENECT_LOG_SPEW,
            LogLevel::Flood     => freenect_loglevel::FREENECT_LOG_FLOOD,
        }
    }

    fn from_lowlevel(lvl: freenect_loglevel) -> LogLevel {
        match lvl {
            freenect_loglevel::FREENECT_LOG_FATAL   => LogLevel::Fatal,
            freenect_loglevel::FREENECT_LOG_ERROR   => LogLevel::Error,
            freenect_loglevel::FREENECT_LOG_WARNING => LogLevel::Warning,
            freenect_loglevel::FREENECT_LOG_NOTICE  => LogLevel::Notice,
            freenect_loglevel::FREENECT_LOG_INFO    => LogLevel::Info,
            freenect_loglevel::FREENECT_LOG_DEBUG   => LogLevel::Debug,
            freenect_loglevel::FREENECT_LOG_SPEW    => LogLevel::Spew,
            freenect_loglevel::FREENECT_LOG_FLOOD   => LogLevel::Flood,
        }
    }
}

#[derive(Debug)]
pub enum Resolution {
    Low,
    Medium,
    High,
}

impl Resolution {
    fn to_lowlevel(&self) -> freenect_resolution {
        match *self {
            Resolution::Low     => freenect_resolution::FREENECT_RESOLUTION_LOW,
            Resolution::Medium  => freenect_resolution::FREENECT_RESOLUTION_MEDIUM,
            Resolution::High    => freenect_resolution::FREENECT_RESOLUTION_HIGH,
        }
    }

    fn from_lowlevel(res: &freenect_resolution) -> Resolution {
        match *res {
            freenect_resolution::FREENECT_RESOLUTION_LOW    => Resolution::Low,
            freenect_resolution::FREENECT_RESOLUTION_MEDIUM => Resolution::Medium,
            freenect_resolution::FREENECT_RESOLUTION_HIGH   => Resolution::High,
            _ => panic!("Unknown freenect_resolution enum")
        }
    }
}

#[derive(Debug)]
pub enum VideoFormat {
    Rgb,
    Bayer,
    Ir8Bit,
    Ir10Bit,
    Ir10BitPacked,
    YuvRgb,
    YuvRaw,
}

impl VideoFormat {
    fn to_lowlevel(&self) -> freenect_video_format {
        match *self {
            VideoFormat::Rgb            => freenect_video_format::FREENECT_VIDEO_RGB,
            VideoFormat::Bayer          => freenect_video_format::FREENECT_VIDEO_BAYER,
            VideoFormat::Ir8Bit         => freenect_video_format::FREENECT_VIDEO_IR_8BIT,
            VideoFormat::Ir10Bit        => freenect_video_format::FREENECT_VIDEO_IR_10BIT,
            VideoFormat::Ir10BitPacked  => freenect_video_format::FREENECT_VIDEO_IR_10BIT_PACKED,
            VideoFormat::YuvRgb         => freenect_video_format::FREENECT_VIDEO_YUV_RGB,
            VideoFormat::YuvRaw         => freenect_video_format::FREENECT_VIDEO_YUV_RAW,
        }
    }

    fn from_lowlevel_int(i: int32_t) -> VideoFormat {
        match i {
            0 => VideoFormat::Rgb,
            1 => VideoFormat::Bayer,
            2 => VideoFormat::Ir8Bit,
            3 => VideoFormat::Ir10Bit,
            4 => VideoFormat::Ir10BitPacked,
            5 => VideoFormat::YuvRgb,
            6 => VideoFormat::YuvRaw,
            _ => panic!("Unknown freenect_video_format enum"),
        }
    }
}

#[derive(Debug)]
pub enum DepthFormat {
    _11Bit,
    _10Bit,
    _11BitPacked,
    _10BitPacked,
    Registered,
    Mm
}

impl DepthFormat {
    fn to_lowlevel(&self) -> freenect_depth_format {
        match *self {
            DepthFormat::_11Bit         => freenect_depth_format::FREENECT_DEPTH_11BIT,
            DepthFormat::_10Bit         => freenect_depth_format::FREENECT_DEPTH_10BIT,
            DepthFormat::_11BitPacked   => freenect_depth_format::FREENECT_DEPTH_11BIT_PACKED,
            DepthFormat::_10BitPacked   => freenect_depth_format::FREENECT_DEPTH_10BIT_PACKED,
            DepthFormat::Registered     => freenect_depth_format::FREENECT_DEPTH_REGISTERED,
            DepthFormat::Mm             => freenect_depth_format::FREENECT_DEPTH_MM,
        }
    }

    fn from_lowlevel_int(i: int32_t) -> DepthFormat {
        match i {
            0 => DepthFormat::_11Bit,
        	1 => DepthFormat::_10Bit,
        	2 => DepthFormat::_11BitPacked,
        	3 => DepthFormat::_10BitPacked,
        	4 => DepthFormat::Registered,
        	5 => DepthFormat::Mm,
        	_ => panic!("Unknown freenect_depth_format enum"),
        }
    }
}

#[derive(Debug)]
enum FrameModeFormat {
    Video(VideoFormat),
    Depth(DepthFormat),
}

#[derive(Debug)]
pub struct FrameMode {
    reserved: uint32_t, // Need to track contents of underlying freenect struct
    resolution: Resolution,
    format: FrameModeFormat,
    bytes: i32,
    width: i16,
    height: i16,
    data_bits_per_pixel: i8,
    padding_bits_per_pixel: i8,
    framerate: i8,
    is_valid: bool,
}

impl FrameMode {
    fn to_lowlevel(&self) -> freenect_frame_mode {
        freenect_frame_mode {
            reserved: self.reserved,
            resolution: self.resolution.to_lowlevel(),
            dummy: match self.format {
                FrameModeFormat::Video(ref x) => x.to_lowlevel() as int32_t,
                FrameModeFormat::Depth(ref y) => y.to_lowlevel() as int32_t,
            },
            bytes: self.bytes,
            width: self.width,
            height: self.height,
            data_bits_per_pixel: self.data_bits_per_pixel,
            padding_bits_per_pixel: self.padding_bits_per_pixel,
            framerate: self.framerate,
            is_valid: if self.is_valid { 1 } else { 0 },
        }
    }

    fn to_lowlevel_video(&self) -> Option<freenect_frame_mode> {
        match self.format {
            FrameModeFormat::Video(ref x) => Some(self.to_lowlevel()),
            FrameModeFormat::Depth(_) => None,
        }
    }

    fn to_lowlevel_depth(&self) -> Option<freenect_frame_mode> {
        match self.format {
            FrameModeFormat::Depth(ref x) => Some(self.to_lowlevel()),
            FrameModeFormat::Video(_) => None,
        }
    }

    fn from_lowlevel(mode: &freenect_frame_mode, fmt: FrameModeFormat) -> FrameMode {
        FrameMode {
            reserved: mode.reserved,
            resolution: Resolution::from_lowlevel(&mode.resolution),
            format: fmt,
            bytes: mode.bytes as i32,
            width: mode.width as i16,
            height: mode.height as i16,
            data_bits_per_pixel: mode.data_bits_per_pixel as i8,
            padding_bits_per_pixel: mode.padding_bits_per_pixel as i8,
            framerate: mode.framerate as i8,
            is_valid: if mode.is_valid > 0 { true } else { false },
        }
    }

    fn from_lowlevel_video(mode: &freenect_frame_mode) -> FrameMode {
        FrameMode::from_lowlevel(mode, FrameModeFormat::Video(VideoFormat::from_lowlevel_int(mode.dummy)))
    }

    fn from_lowlevel_depth(mode: &freenect_frame_mode) -> FrameMode {
        FrameMode::from_lowlevel(mode, FrameModeFormat::Depth(DepthFormat::from_lowlevel_int(mode.dummy)))
    }
}


bitflags! {
    flags DeviceFlags: u32 {
        const DEVICE_MOTOR  = freenect_device_flags::FREENECT_DEVICE_MOTOR  as u32,
        const DEVICE_CAMERA = freenect_device_flags::FREENECT_DEVICE_CAMERA as u32,
        const DEVICE_AUDIO  = freenect_device_flags::FREENECT_DEVICE_AUDIO  as u32,
    }
}

#[derive(Debug)]
pub struct DeviceAttributes {
    pub camera_serial: String,
}

#[derive(Debug)]
struct InnerContext {
    ctx: *mut freenect_context,
}

// InnerContext separated from main Context so that 'Device' handles can hold a reference to the
// InnerContext to prevent premature release. Could also use lifetimes (probably) to statically
// enforce this.
impl InnerContext {
    fn new() -> FreenectResult<InnerContext> {
        let mut ctx = InnerContext{ctx: ptr::null_mut()};

        match unsafe { freenect_init(&mut ctx.ctx, ptr::null_mut()) } {
            0 => {
                if ctx.ctx != ptr::null_mut() {
                    Ok(ctx)
                } else {
                    Err(FreenectError::NullPtr)
                }
            },
            x => Err(FreenectError::LibraryReturnCode(x)),
        }
    }
}

impl Drop for InnerContext {
    fn drop(&mut self) {
        let ret = unsafe { freenect_shutdown(self.ctx) };

        if ret < 0 {
            panic!(ret)
        }
    }
}

#[derive(Debug)]
pub struct Context {
    ctx: Rc<InnerContext>,
}

impl Context {
    pub fn new() -> FreenectResult<Context> {
        let inner_ctx = try!(InnerContext::new());

        Ok(Context{ctx: Rc::new(inner_ctx)})
    }

    pub fn set_log_level(&mut self, level: LogLevel) {
        unsafe { freenect_set_log_level(self.ctx.ctx, level.to_lowlevel()); }
    }

    pub fn process_events(&mut self) -> FreenectResult<()> {
        match unsafe { freenect_process_events(self.ctx.ctx) } {
            0 => Ok(()),
            x => Err(FreenectError::LibraryReturnCode(x)),
        }
    }

    // FIXME: Implement process_events with timeout

    pub fn num_devices(&mut self) -> FreenectResult<u32> {
        let ret = unsafe { freenect_num_devices(self.ctx.ctx) };
        if ret < 0 {
            Err(FreenectError::LibraryReturnCode(ret))
        } else {
            Ok(ret as u32)
        }
    }

    pub fn list_device_attributes(&mut self) -> FreenectResult<Vec<DeviceAttributes>> {
        let mut lowlevel_list: *mut freenect_device_attributes = ptr::null_mut();

        let ret = unsafe { freenect_list_device_attributes(self.ctx.ctx, &mut lowlevel_list) };
        if ret < 0 {
            return Err(FreenectError::LibraryReturnCode(ret));
        }

        let mut device_list: Vec<DeviceAttributes> = Vec::new();

        let mut curr_item = lowlevel_list;
        while curr_item != ptr::null_mut() {
            let serial_cstr = unsafe { ffi::CStr::from_ptr((*curr_item).camera_serial) };
            let serial = String::from_utf8_lossy(serial_cstr.to_bytes()).to_string();

            device_list.push(DeviceAttributes{camera_serial: serial});
            unsafe { curr_item = (*curr_item).next };
        }

        unsafe { freenect_free_device_attributes(lowlevel_list) };

        Ok(device_list)
    }

    // Internal use only
    fn select_subdevices(&mut self, subdevs: DeviceFlags) {
        unsafe { freenect_select_subdevices(self.ctx.ctx, subdevs.bits) };
    }

    // Internal use only
    fn enabled_subdevices(&mut self) -> DeviceFlags {
        let ret = unsafe { freenect_enabled_subdevices(self.ctx.ctx) };

        return DeviceFlags::from_bits(ret as u32).unwrap();
    }

    pub fn open_device(&mut self, index: u32, subdevs: DeviceFlags) -> FreenectResult<Device> {
        let mut dev: *mut freenect_device = ptr::null_mut();

        self.select_subdevices(subdevs);

        let ret = unsafe { freenect_open_device(self.ctx.ctx, &mut dev, index as i32) };
        if ret < 0 {
            return Err(FreenectError::LibraryReturnCode(ret))
        }

        return Ok(Device::from_raw_device(self.ctx.clone(), dev, self.enabled_subdevices()));
    }

    pub fn open_device_by_camera_serial(&mut self, serial: &str, subdevs: DeviceFlags) -> FreenectResult<Device> {
        let mut dev: *mut freenect_device = ptr::null_mut();

        let serial_cstring = ffi::CString::new(serial).unwrap();

        self.select_subdevices(subdevs);

        let ret = unsafe { freenect_open_device_by_camera_serial(self.ctx.ctx, &mut dev, serial_cstring.as_ptr()) };
        if ret < 0 {
            return Err(FreenectError::LibraryReturnCode(ret))
        }

        return Ok(Device::from_raw_device(self.ctx.clone(), dev, self.enabled_subdevices()));
    }
}

struct InnerDevice {
    dev: *mut freenect_device,
}

impl Drop for InnerDevice {
    fn drop(&mut self) {
        let ret = unsafe { freenect_close_device(self.dev) };

        if ret < 0 {
            panic!(ret)
        }
    }
}

impl InnerDevice {
    fn get_current_video_mode(&mut self) -> FrameMode {
        let lowlevel_video_mode = unsafe { freenect_get_current_video_mode(self.dev) };
        FrameMode::from_lowlevel_video(&lowlevel_video_mode)
    }

    fn get_current_depth_mode(&mut self) -> FrameMode {
        let lowlevel_depth_mode = unsafe { freenect_get_current_depth_mode(self.dev) };
        FrameMode::from_lowlevel_depth(&lowlevel_depth_mode)
    }
}

pub struct Device {
    ctx: Rc<InnerContext>, // Handle to prevent underlying context being free'd before device
    dev: Rc<RefCell<InnerDevice>>,
    ch: Box<ClosureHolder>,
    pub motor:  Option<MotorSubdevice>,
    pub camera: Option<CameraSubdevice>,
    pub audio:  Option<AudioSubdevice>,
}

impl Device {
    fn from_raw_device(ctx: Rc<InnerContext>, dev: *mut freenect_device, subdevs: DeviceFlags) -> Device {
        let mut inner_dev = Rc::new(RefCell::new(InnerDevice{dev: dev}));

        let mut dev = Device {
            ctx: ctx,
            dev: inner_dev.clone(),
            ch: Box::new(ClosureHolder {
                dev: inner_dev.clone(),
                depth_cb: None,
                video_cb: None,
            }),
            motor:  if subdevs.contains(DEVICE_MOTOR)  { Some(MotorSubdevice{dev: inner_dev.clone()})  } else { None },
            camera: if subdevs.contains(DEVICE_CAMERA) {
                        Some(CameraSubdevice{
                            dev: inner_dev.clone(),
                            })
                    } else {
                        None
                    },
            audio:  if subdevs.contains(DEVICE_AUDIO)  { Some(AudioSubdevice{dev: inner_dev.clone()})  } else { None },
        };

        // Register all callbacks. We'll let Rust code decide if a user callback should be called.
        unsafe {
            let mut inner_dev = dev.dev.borrow_mut();

            freenect_set_user(inner_dev.dev, std::mem::transmute(&mut *dev.ch));

            freenect_set_depth_callback(inner_dev.dev, Device::depth_cb_trampoline);
            freenect_set_video_callback(inner_dev.dev, Device::video_cb_trampoline);
        }

        return dev;
    }

    extern "C" fn depth_cb_trampoline(dev: *mut freenect_device, depth: *mut c_void, timestamp: uint32_t) {
        unsafe {
            let ch = freenect_get_user(dev) as *mut ClosureHolder;

            // Callback provides no information on frame buffer length. Retrieve the length by
            // directly asking for the current mode information
            let mode = (*ch).dev.borrow_mut().get_current_depth_mode();

            let frame = slice::from_raw_parts_mut(depth as *mut u8, mode.bytes as usize);
            let timestamp = timestamp as u32;

            match (*ch).depth_cb {
                Some(ref mut cb) => cb(frame, timestamp),
                None => return,
            };
        }
    }

    extern "C" fn video_cb_trampoline(dev: *mut freenect_device, video: *mut c_void, timestamp: uint32_t) {
        unsafe {
            let ch = freenect_get_user(dev) as *mut ClosureHolder;

            // Callback provides no information on frame buffer length. Retrieve the length by
            // directly asking for the current mode information
            let mode = (*ch).dev.borrow_mut().get_current_video_mode();

            let frame = slice::from_raw_parts_mut(video as *mut u8, mode.bytes as usize);
            let timestamp = timestamp as u32;

            match (*ch).video_cb {
                Some(ref mut cb) => cb(frame, timestamp),
                None => return,
            };
        }
    }

    pub fn set_depth_callback(&mut self, cb: Option<Box<FnMut(&mut [u8], u32)>>) {
        self.ch.depth_cb = cb;
    }

    pub fn set_video_callback(&mut self, cb: Option<Box<FnMut(&mut [u8], u32)>>) {
        self.ch.video_cb = cb;
    }

    pub fn start_depth(&mut self) -> FreenectResult<()> {
        let ret = unsafe { freenect_start_depth(self.dev.borrow_mut().dev)};

        if ret == 0 {
            Ok(())
        } else {
            Err(FreenectError::LibraryReturnCode(ret))
        }
    }

    pub fn start_video(&mut self) -> FreenectResult<()> {
        let ret = unsafe { freenect_start_video(self.dev.borrow_mut().dev)};

        if ret == 0 {
            Ok(())
        } else {
            Err(FreenectError::LibraryReturnCode(ret))
        }
    }

    pub fn stop_depth(&mut self) -> FreenectResult<()> {
        let ret = unsafe { freenect_stop_depth(self.dev.borrow_mut().dev)};

        if ret == 0 {
            Ok(())
        } else {
            Err(FreenectError::LibraryReturnCode(ret))
        }
    }

    pub fn stop_video(&mut self) -> FreenectResult<()> {
        let ret = unsafe { freenect_stop_video(self.dev.borrow_mut().dev)};

        if ret == 0 {
            Ok(())
        } else {
            Err(FreenectError::LibraryReturnCode(ret))
        }
    }

    pub fn get_current_video_mode(&mut self) -> FrameMode {
        self.dev.borrow_mut().get_current_video_mode()
    }

    pub fn set_video_mode(&mut self, mode: FrameMode) -> FreenectResult<()> {
        let mut lowlevel_video_mode = try!(mode.to_lowlevel_video().ok_or(FreenectError::FrameFormatMismatch));
        unsafe { freenect_set_video_mode(self.dev.borrow_mut().dev, lowlevel_video_mode) };
        Ok(())
    }

    // pub fn freenect_get_video_mode(mode_num: c_int) -> freenect_frame_mode;
}

// Exists so it can be boxed (therefore fixing its memory address) and have its address handed as a
// C callback userdata  void pointer
struct ClosureHolder {
    dev: Rc<RefCell<InnerDevice>>,
    depth_cb: Option<Box<FnMut(&mut [u8], u32)>>,
    video_cb: Option<Box<FnMut(&mut [u8], u32)>>,
}

pub struct MotorSubdevice {
    dev: Rc<RefCell<InnerDevice>>,
}

pub struct CameraSubdevice {
    dev: Rc<RefCell<InnerDevice>>,
}

pub struct AudioSubdevice {
    dev: Rc<RefCell<InnerDevice>>,
}

pub fn supported_subdevices() -> DeviceFlags {
    let bits = unsafe { freenect_supported_subdevices() as u32 };
    return DeviceFlags::from_bits(bits).unwrap();
}
