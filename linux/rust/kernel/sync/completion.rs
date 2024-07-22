use crate::{bindings,  Opaque};
use core::marker::PhantomPinned;


/// A wrapper around a kernel completion object.
pub struct Completion {
    completion: Opaque<bindings::completion>,
}

impl Completion {
    /// The caller must call `completion_init!` before using the conditional variable.
    pub const unsafe fn new() -> Self {
        Self {
            completion: Opaque::uninit(),
        }
    }

    /// Initialise the completion.
    pub fn init(&self) {
        unsafe {
            bindings::init_completion(self.completion())
        };
    }

    /// Wait for the completion to complete.
    pub fn wait_for_completion(&self) {
        unsafe { bindings::wait_for_completion(self.completion.get()) }
    }

    /// Complete the completion.
    pub fn complete(&self) {
        unsafe { bindings::complete(self.completion.get()) }
    }

    /// Get the completion pointer.
    pub fn completion(&self) -> *mut bindings::completion {
        self.completion.get()
    }
}
