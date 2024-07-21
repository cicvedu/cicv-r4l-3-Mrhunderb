// SPDX-License-Identifier: GPL-2.0

//! Rust character device sample.

use core::result::Result::Err;

use kernel::prelude::*;
use kernel::{chrdev, file};
use kernel::io_buffer::IoBufferWriter;
use kernel::io_buffer::IoBufferReader;
use kernel::task::Task;
use kernel::sync::{CondVar, Mutex, Arc};


module! {
    type: CompletionChrDev,
    name: "rust_completion",
    author: "xforcevesa",
    description: "Rust character device sample",
    license: "GPL",
}

struct CompletionState {
    completed: bool,
    cond: CondVar,
}

struct CompletionFile {
    #[allow(dead_code)]
    state: Arc<Mutex<CompletionState>>,
}

#[vtable]
impl file::Operations for CompletionFile {
    type Data = Box<Self>;

    fn open(_shared: &(), _file: &file::File) -> Result<Box<Self>> {
        pr_info!("open() is invoked\n");
        Ok(
            Box::try_new(CompletionFile {
                state: Arc::new(unsafe { Mutex::new(CompletionState {
                    completed: false,
                    cond: CondVar::new(),
                }) }),
            })?
        )
    }


    fn read(this: &Self, _file: &file::File, writer: &mut impl IoBufferWriter, _offset: u64) -> Result<usize> {
        pr_info!("read() is invoked\n");

        let task = Task::current();

        pr_info!("process {} is going to sleep\n",task.pid());
        let mut guard = this.state.lock();
        while !guard.completed {
            guard = guard.cond.wait(&mut guard);
        }
        guard.completed = false; // Reset for next use
        drop(guard);
        pr_info!("awoken {}\n", task.pid());
        Ok(0)
    }

    fn write(this: &Self, _file: &file::File, reader: &mut impl IoBufferReader, offset: u64) -> Result<usize> {
        pr_info!("write() is invoked\n");

        let task = Task::current();
        pr_info!("process {} awakening the readers...\n", task.pid());
        pr_info!("data.len() = {}\n", reader.len());

        let mut guard = this.state.lock();
        guard.completed = true;
        guard.cond.notify_all();
        
        Ok(reader.len())
    }

    fn release(_data: Self::Data, _file: &file::File) {
        pr_info!("release() is invoked\n");
    }
}

struct CompletionChrDev {
    _dev: Pin<Box<chrdev::Registration<1>>>,
}

impl kernel::Module for CompletionChrDev {
    fn init(name: &'static CStr, module: &'static ThisModule) -> Result<Self> {
        pr_info!("completion_example is loaded\n");

        let mut chrdev_reg = chrdev::Registration::new_pinned(name, 0, module)?;

        // Register the same kind of device twice, we're just demonstrating
        // that you can use multiple minors. There are two minors in this case
        // because its type is `chrdev::Registration<2>`
        chrdev_reg.as_mut().register::<CompletionFile>()?;

        Ok(CompletionChrDev { _dev: chrdev_reg })
    }
}

impl Drop for CompletionChrDev {
    fn drop(&mut self) {
        pr_info!("completion_example is unloaded\n");
    }
}