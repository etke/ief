use std::path::Path;
use std::thread;

use binaryninja::binaryninjacore_sys::{BNBinaryView, BNShowMarkdownReport};
use binaryninja::binaryview::{BinaryView, BinaryViewExt};
use binaryninja::command::{
    register, register_for_address, register_for_function,
};
use binaryninja::function::Function;
use binaryninja::interaction::{
    get_directory_name_input, get_text_line_input,
};
use binaryninja::logger;
use binaryninja::string::BnString;
use log::{error, LevelFilter};

use binaryninja::backgroundtask::BackgroundTask;

use crate::walk::walk;
use crate::SymbolType;

fn find_export(task: &mut BackgroundTask, view: &BinaryView, addr: u64) {
    let mut defaultdir =
        Path::new(view.metadata().filename().as_str()).to_path_buf();
    defaultdir.pop();
    if let Some(directory) = get_directory_name_input(
        "Directory to search",
        defaultdir.to_str().unwrap(),
    ) {
        let name = view.symbol_by_address(addr).unwrap().short_name();
        let sep = format!("{:-<80}", "\n");
        let mut markdown = format!(
            "# IEF Results\n\n{}\n\
            binaries in ```{}``` with exported symbol ```{}```\n\n\n",
            sep,
            directory.display(),
            name
        );
        task.set_progress_text(format!(
            "IEF: searching for export \"{}\" in {}",
            name,
            directory.display()
        ));
        for entry in walk(&directory, &SymbolType::Export, name.as_str()) {
            markdown += format!("\t* {}\n", entry).as_str();
        }
        let contents = BnString::new(markdown);
        let title = BnString::new(format!("IEF - {}", name));

        unsafe {
            // You are about to experience the awe and mystery which
            // reaches from the inner mind to the outer limits.
            BNShowMarkdownReport(
                std::ptr::null::<BNBinaryView>() as *mut _,
                title.as_cstr().as_ptr(),
                contents.as_cstr().as_ptr(),
                contents.as_cstr().as_ptr(),
            );
        }
    }
}

fn find_export_bg(view: &BinaryView, addr: u64) {
    let view_ref = view.to_owned();
    thread::spawn(move || {
        if let Ok(mut task) = BackgroundTask::new(
            "IEF: Finding binaries with exported symbol...",
            true,
        ) {
            find_export(&mut task, &view_ref, addr);
            task.finish();
        }
    });
}

fn find_import(task: &mut BackgroundTask, view: &BinaryView, func: &Function) {
    let mut defaultdir =
        Path::new(view.metadata().filename().as_str()).to_path_buf();
    defaultdir.pop();
    if let Some(directory) = get_directory_name_input(
        "Directory to search",
        defaultdir.to_str().unwrap(),
    ) {
        let name = func.symbol().short_name();
        let sep = format!("{:-<80}", "\n");
        let mut markdown = format!(
            "# IEF Results\n\n{}\n\
            binaries in ```{}``` with imported symbol ```{}```\n\n\n",
            sep,
            directory.display(),
            name
        );
        task.set_progress_text(format!(
            "IEF: searching for import \"{}\" in {}",
            name,
            directory.display()
        ));

        for entry in walk(&directory, &SymbolType::Import, name.as_str()) {
            markdown += format!("\t* {})\n", entry).as_str();
        }
        let contents = BnString::new(markdown);
        let title = BnString::new(format!("IEF - {}", name));

        unsafe {
            // You are about to experience the awe and mystery which
            // reaches from the inner mind to the outer limits.
            BNShowMarkdownReport(
                std::ptr::null::<BNBinaryView>() as *mut _,
                title.as_cstr().as_ptr(),
                contents.as_cstr().as_ptr(),
                contents.as_cstr().as_ptr(),
            );
        }
    }
}

fn find_import_bg(view: &BinaryView, func: &Function) {
    let view_ref = view.to_owned();
    let func_ref = func.to_owned();
    thread::spawn(move || {
        if let Ok(mut task) = BackgroundTask::new(
            "IEF: Finding binaries with imported symbol...",
            true,
        ) {
            find_import(&mut task, &view_ref, &func_ref);
            task.finish();
        }
    });
}

fn find_library(task: &mut BackgroundTask, view: &BinaryView) {
    let mut defaultdir =
        Path::new(view.metadata().filename().as_str()).to_path_buf();
    defaultdir.pop();
    if let Some(directory) = get_directory_name_input(
        "Directory to search",
        defaultdir.to_str().unwrap(),
    ) {
        if let Some(name) = get_text_line_input(
            "Library name (can be partial)",
            "IEF - Library search",
        ) {
            let sep = format!("{:-<80}", "\n");
            let mut markdown = format!(
                "# IEF Results\n\n{}\nbinaries in ```{}``` with imported \
                library name containing ```{}```\n\n",
                sep,
                directory.display(),
                name
            );
            task.set_progress_text(format!(
                "IEF: searching for binaries that import library \"{}\" in {}",
                name,
                directory.display()
            ));
            for entry in walk(&directory, &SymbolType::Library, name.as_str())
            {
                markdown += format!(
                    // "* [{}](binaryninja:file://{})\n",
                    "* [{}]({})\n",
                    entry, entry
                )
                .as_str();
            }
            let contents = BnString::new(markdown);
            let title = BnString::new(format!("IEF - {}", name));

            unsafe {
                // You are about to experience the awe and mystery which
                // reaches from the inner mind to the outer limits.
                BNShowMarkdownReport(
                    std::ptr::null::<BNBinaryView>() as *mut _,
                    title.as_cstr().as_ptr(),
                    contents.as_cstr().as_ptr(),
                    contents.as_cstr().as_ptr(),
                );
            }
        }
    }
}

fn find_library_bg(view: &BinaryView) {
    let view_ref = view.to_owned();
    thread::spawn(move || {
        if let Ok(mut task) =
            BackgroundTask::new("IEF: Finding library in background...", true)
        {
            find_library(&mut task, &view_ref);
            task.finish();
        }
    });
}

#[no_mangle]
pub extern "C" fn UIPluginInit() -> bool {
    if logger::init(LevelFilter::Info).is_err() {
        error!("Initialization failed, skipping IEF plugin...");
        return false;
    }

    register(
        "Import Export Find\\Find binaries that import library named",
        "find binaries that import a library named.",
        find_library_bg,
    );

    register_for_address(
        "Import Export Find\\Find binaries that export symbol at address",
        "find binaries that export the current function.",
        find_export_bg,
    );

    register_for_function(
        "Import Export Find\\Find binaries that import current function",
        "find binaries that import the current function.",
        find_import_bg,
    );

    true
}
