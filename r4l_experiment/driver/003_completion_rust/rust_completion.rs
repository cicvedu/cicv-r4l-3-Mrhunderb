// SPDX-License-Identifier: GPL-2.0

//! Rust character device sample.

use core::result::Result::Err;

use kernel::prelude::*;
use kernel::{chrdev, file};
use kernel::io_buffer::IoBufferWriter;
use kernel::io_buffer::IoBufferReader;
use kernel::task::Task;
use kernel::sync::Completion;


module! {
    type: CompletionChrDev,
    name: "rust_completion",
    author: "Tester",
    description: "Rust completion sample",
    license: "GPL",
}

static COMPLETION: Completion = unsafe { Completion::new() };

struct CompletionFile {
    completed: &'static Completion,
}

#[vtable]
impl file::Operations for CompletionFile {
    type Data = Box<Self>;

    fn open(_shared: &(), _file: &file::File) -> Result<Box<Self>> {
        pr_info!("open() is invoked\n");
        Ok(
            Box::try_new(CompletionFile {
                completed: &COMPLETION,
            })?
        )
    }


    fn read(this: &Self, _file: &file::File, _writer: &mut impl IoBufferWriter, _offset: u64) -> Result<usize> {
        pr_info!("read() is invoked\n");

        let task = Task::current();

        pr_info!("process {} is going to sleep\n",task.pid());
        this.completed.wait_for_completion();
        pr_info!("awoken {}\n", task.pid());
        Ok(0)
    }

    fn write(this: &Self, _file: &file::File, reader: &mut impl IoBufferReader, _offset: u64) -> Result<usize> {
        pr_info!("write() is invoked\n");

        let task = Task::current();
        pr_info!("process {} awakening the readers...\n", task.pid());
        pr_info!("data.len() = {}\n", reader.len());
        this.completed.complete();
        
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

        COMPLETION.init();
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