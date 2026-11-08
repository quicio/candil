//! Integración mínima con AppKit para el icono de la aplicación en macOS.
//!
//! `ViewportBuilder::with_icon` cambia el icono de la ventana, pero macOS usa
//! `NSApplication.applicationIconImage` para el Dock y el selector de apps.
//! Sin esto, un binario lanzado desde la terminal muestra el icono genérico de
//! ejecutable. No agrega dependencias: usa el runtime Objective-C que AppKit ya
//! carga a través de winit/eframe.

use std::ffi::{c_char, c_void, CString};

#[link(name = "AppKit", kind = "framework")]
extern "C" {
    fn objc_getClass(name: *const c_char) -> *mut c_void;
    fn sel_registerName(name: *const c_char) -> *mut c_void;
    fn objc_msgSend();
}

type Msg0 = unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void;
type Msg1 = unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> *mut c_void;
type Msg1Void = unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void);
type Msg2 = unsafe extern "C" fn(*mut c_void, *mut c_void, *const c_void, usize) -> *mut c_void;

unsafe fn class(name: &str) -> *mut c_void {
    let name = CString::new(name).expect("nombre de clase Objective-C sin NUL");
    objc_getClass(name.as_ptr())
}

unsafe fn selector(name: &str) -> *mut c_void {
    let name = CString::new(name).expect("selector Objective-C sin NUL");
    sel_registerName(name.as_ptr())
}

unsafe fn send0(receiver: *mut c_void, selector: *mut c_void) -> *mut c_void {
    let f: Msg0 = std::mem::transmute(objc_msgSend as *const ());
    f(receiver, selector)
}

unsafe fn send1(receiver: *mut c_void, selector: *mut c_void, arg: *mut c_void) -> *mut c_void {
    let f: Msg1 = std::mem::transmute(objc_msgSend as *const ());
    f(receiver, selector, arg)
}

unsafe fn send1_void(receiver: *mut c_void, selector: *mut c_void, arg: *mut c_void) {
    let f: Msg1Void = std::mem::transmute(objc_msgSend as *const ());
    f(receiver, selector, arg);
}

unsafe fn send2(
    receiver: *mut c_void,
    selector: *mut c_void,
    a: *const c_void,
    b: usize,
) -> *mut c_void {
    let f: Msg2 = std::mem::transmute(objc_msgSend as *const ());
    f(receiver, selector, a, b)
}

/// Fija el icono de la app para el Dock y el selector de aplicaciones.
pub fn set_app_icon() {
    // PNG de 256 px ya normalizado; AppKit lo escala para el Dock.
    let bytes = include_bytes!("../assets/candil-icon-256.png");
    // SAFETY: se llama en el hilo principal, después de que AppKit creó
    // NSApplication. Los objetos se pasan al runtime Objective-C y AppKit
    // retiene la imagen al asignarla.
    unsafe {
        let data = send2(
            class("NSData"),
            selector("dataWithBytes:length:"),
            bytes.as_ptr() as *const c_void,
            bytes.len(),
        );
        if data.is_null() {
            return;
        }
        let allocated = send0(class("NSImage"), selector("alloc"));
        if allocated.is_null() {
            return;
        }
        let image = send1(allocated, selector("initWithData:"), data);
        if image.is_null() {
            return;
        }
        let app = send0(class("NSApplication"), selector("sharedApplication"));
        if app.is_null() {
            return;
        }
        send1_void(app, selector("setApplicationIconImage:"), image);
    }
}
