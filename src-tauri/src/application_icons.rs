//! Lazy native Shell icon extraction. Every COM/GDI resource stays on the
//! blocking worker that created it, and all bitmap/DC handles are released.

use super::{
    LaunchTarget,
    platform::{ComApartment, wide_path},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use std::{ffi::c_void, mem::size_of};
use windows::{
    Win32::{
        Foundation::SIZE,
        Graphics::Gdi::{
            BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, DeleteObject, GetDC,
            GetDIBits, GetObjectW, HBITMAP, HDC, HGDIOBJ, ReleaseDC,
        },
        UI::Shell::{
            FOLDERID_AppsFolder, IShellItemImageFactory, KF_FLAG_DEFAULT,
            SHCreateItemFromParsingName, SHCreateItemInKnownFolder, SIIGBF_BIGGERSIZEOK,
            SIIGBF_ICONONLY,
        },
    },
    core::PCWSTR,
};

const ICON_SIZE: i32 = 64;
const MAX_BITMAP_EDGE: i32 = 256;
// Base64 overhead keeps the complete data URL below the renderer's 96 KiB cap.
const MAX_PNG_BYTES: usize = 64 * 1024;

struct OwnedBitmap(HBITMAP);

impl Drop for OwnedBitmap {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteObject(HGDIOBJ(self.0.0));
        }
    }
}

struct ScreenDc(HDC);

impl Drop for ScreenDc {
    fn drop(&mut self) {
        unsafe {
            let _ = ReleaseDC(None, self.0);
        }
    }
}

pub(super) fn extract(target: &LaunchTarget) -> Option<String> {
    let _apartment = ComApartment::initialize().ok()?;
    let factory: IShellItemImageFactory = match target {
        LaunchTarget::Shortcut(path) => {
            let path = wide_path(path).ok()?;
            unsafe { SHCreateItemFromParsingName(PCWSTR(path.as_ptr()), None) }.ok()?
        }
        LaunchTarget::WindowsApp(aumid) => {
            let item: Vec<u16> = aumid.encode_utf16().chain(std::iter::once(0)).collect();
            unsafe {
                SHCreateItemInKnownFolder(
                    &FOLDERID_AppsFolder,
                    KF_FLAG_DEFAULT,
                    PCWSTR(item.as_ptr()),
                )
            }
            .ok()?
        }
    };
    // ICONONLY avoids opening documents to extract content thumbnails. This
    // work never runs on the Tauri event thread or during catalogue discovery.
    let bitmap = OwnedBitmap(
        unsafe {
            factory.GetImage(
                SIZE {
                    cx: ICON_SIZE,
                    cy: ICON_SIZE,
                },
                SIIGBF_ICONONLY | SIIGBF_BIGGERSIZEOK,
            )
        }
        .ok()?,
    );
    let mut object = BITMAP::default();
    if unsafe {
        GetObjectW(
            HGDIOBJ(bitmap.0.0),
            size_of::<BITMAP>() as i32,
            Some((&mut object as *mut BITMAP).cast::<c_void>()),
        )
    } == 0
    {
        return None;
    }
    let width = object.bmWidth;
    let height = object.bmHeight.checked_abs()?;
    if !(1..=MAX_BITMAP_EDGE).contains(&width) || !(1..=MAX_BITMAP_EDGE).contains(&height) {
        return None;
    }
    let dc = ScreenDc(unsafe { GetDC(None) });
    if dc.0.0.is_null() {
        return None;
    }
    let mut rgba = vec![0u8; width as usize * height as usize * 4];
    let mut info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let rows = unsafe {
        GetDIBits(
            dc.0,
            bitmap.0,
            0,
            height as u32,
            Some(rgba.as_mut_ptr().cast::<c_void>()),
            &mut info,
            DIB_RGB_COLORS,
        )
    };
    if rows != height {
        return None;
    }

    let has_alpha = rgba.chunks_exact(4).any(|pixel| pixel[3] != 0);
    // Legacy 24-bit icons have no alpha information. Keep them visible instead
    // of turning the entire icon transparent after conversion to a PNG.
    if !has_alpha
        && !rgba
            .chunks_exact(4)
            .any(|pixel| pixel[..3].iter().any(|value| *value != 0))
    {
        return None;
    }
    // Shell icon bitmaps normally contain premultiplied BGRA. If any colour is
    // greater than its alpha, this provider returned straight alpha instead.
    let premultiplied = has_alpha
        && rgba
            .chunks_exact(4)
            .all(|pixel| pixel[0] <= pixel[3] && pixel[1] <= pixel[3] && pixel[2] <= pixel[3]);
    for pixel in rgba.chunks_exact_mut(4) {
        pixel.swap(0, 2);
        if !has_alpha {
            pixel[3] = 255;
        } else if premultiplied {
            let alpha = u32::from(pixel[3]);
            for channel in &mut pixel[..3] {
                *channel = if alpha == 0 {
                    0
                } else {
                    ((u32::from(*channel) * 255 + alpha / 2) / alpha).min(255) as u8
                };
            }
        }
    }
    let mut encoded = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut encoded, width as u32, height as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(png::Compression::Fast);
        let mut writer = encoder.write_header().ok()?;
        writer.write_image_data(&rgba).ok()?;
        writer.finish().ok()?;
    }
    if encoded.len() > MAX_PNG_BYTES {
        return None;
    }
    Some(format!(
        "data:image/png;base64,{}",
        STANDARD.encode(encoded)
    ))
}
