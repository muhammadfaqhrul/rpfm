//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// Here are functions that were part of main, but the file got too big to search them efficiently.
// If you need to turn something from main.rs into a function, put the function here.
use super::*;

/// This function opens the PackFile at the provided Path, and sets all the stuff needed, depending
/// on the situation.
/// NOTE: The `game_folder` &str is for when using this function with "MyMods". If you're opening a
/// normal mod, pass an empty &str there.
pub fn open_packfile(
    sender_qt: &Sender<Commands>,
    sender_qt_data: &Sender<Data>,
    receiver_qt: &Rc<RefCell<Receiver<Data>>>,
    pack_file_paths: &[PathBuf],
    app_ui: &AppUI,
    mymod_stuff: &Rc<RefCell<MyModStuff>>,
    mode: &Rc<RefCell<Mode>>,
    game_folder: &str,
    packedfiles_open_in_packedfile_view: &Rc<RefCell<BTreeMap<i32, Rc<RefCell<Vec<String>>>>>>,
    close_global_search_action: *mut Action,
    table_state_data: &Rc<RefCell<BTreeMap<Vec<String>, TableStateData>>>,
) -> Result<()> {

    // Tell the Background Thread to create a new PackFile with the data of one or more from the disk.
    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
    sender_qt.send(Commands::OpenPackFiles).unwrap();
    sender_qt_data.send(Data::VecPathBuf(pack_file_paths.to_vec())).unwrap();

    // Check what response we got.
    match check_message_validity_tryrecv(&receiver_qt) {
    
        // If it's success....
        Data::PackFileUIData(ui_data) => {

            // We choose the right option, depending on our PackFile.
            match ui_data.pfh_file_type {
                PFHFileType::Boot => unsafe { app_ui.change_packfile_type_boot.as_mut().unwrap().set_checked(true); }
                PFHFileType::Release => unsafe { app_ui.change_packfile_type_release.as_mut().unwrap().set_checked(true); }
                PFHFileType::Patch => unsafe { app_ui.change_packfile_type_patch.as_mut().unwrap().set_checked(true); }
                PFHFileType::Mod => unsafe { app_ui.change_packfile_type_mod.as_mut().unwrap().set_checked(true); }
                PFHFileType::Movie => unsafe { app_ui.change_packfile_type_movie.as_mut().unwrap().set_checked(true); }
                PFHFileType::Other(_) => unsafe { app_ui.change_packfile_type_other.as_mut().unwrap().set_checked(true); }
            }

            // Enable or disable these, depending on what data we have in the header.
            unsafe { app_ui.change_packfile_type_data_is_encrypted.as_mut().unwrap().set_checked(ui_data.bitmask.contains(PFHFlags::HAS_ENCRYPTED_DATA)); }
            unsafe { app_ui.change_packfile_type_index_includes_timestamp.as_mut().unwrap().set_checked(ui_data.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS)); }
            unsafe { app_ui.change_packfile_type_index_is_encrypted.as_mut().unwrap().set_checked(ui_data.bitmask.contains(PFHFlags::HAS_ENCRYPTED_INDEX)); }
            unsafe { app_ui.change_packfile_type_header_is_extended.as_mut().unwrap().set_checked(ui_data.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER)); }

            // Set the compression level correctly, because otherwise we may fuckup some files.
            let compression_state = match ui_data.compression_state {
                CompressionState::Enabled => true,
                CompressionState::Partial | CompressionState::Disabled => false,
            };
            unsafe { app_ui.change_packfile_type_data_is_compressed.as_mut().unwrap().set_checked(compression_state); }

            // Update the TreeView.
            update_treeview(
                sender_qt,
                sender_qt_data,
                &receiver_qt,
                &app_ui,
                app_ui.folder_tree_view,
                Some(app_ui.folder_tree_filter),
                app_ui.folder_tree_model,
                TreeViewOperation::Build(false),
            );

            // If it's a "MyMod" (game_folder_name is not empty), we choose the Game selected Depending on it.
            if !game_folder.is_empty() && pack_file_paths.len() == 1 {

                // NOTE: Arena should never be here.
                // Change the Game Selected in the UI.
                match game_folder {
                    "three_kingdoms" => unsafe { app_ui.three_kingdoms.as_mut().unwrap().trigger(); }
                    "warhammer_2" => unsafe { app_ui.warhammer_2.as_mut().unwrap().trigger(); }
                    "warhammer" => unsafe { app_ui.warhammer.as_mut().unwrap().trigger(); }
                    "thrones_of_britannia" => unsafe { app_ui.thrones_of_britannia.as_mut().unwrap().trigger(); }
                    "attila" => unsafe { app_ui.attila.as_mut().unwrap().trigger(); }
                    "rome_2" => unsafe { app_ui.rome_2.as_mut().unwrap().trigger(); }
                    "shogun_2" => unsafe { app_ui.shogun_2.as_mut().unwrap().trigger(); }
                    "napoleon" => unsafe { app_ui.napoleon.as_mut().unwrap().trigger(); }
                    "empire" | _ => unsafe { app_ui.empire.as_mut().unwrap().trigger(); }
                }

                // Set the current "Operational Mode" to `MyMod`.
                set_my_mod_mode(&mymod_stuff, mode, Some(pack_file_paths[0].to_path_buf()));
            }

            // If it's not a "MyMod", we choose the new Game Selected depending on what the open mod id is.
            else {

                // Depending on the Id, choose one game or another.
                match ui_data.pfh_version {

                    // PFH5 is for Warhammer 2/Arena.
                    PFHVersion::PFH5 => {

                        // If the PackFile has the mysterious byte enabled, it's from Arena.
                        if ui_data.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) { 
                            unsafe { app_ui.arena.as_mut().unwrap().trigger(); } 
                        }

                        // Otherwise, it's from Three Kingdoms or Warhammer 2.
                        else { 
                            let game_selected = GAME_SELECTED.lock().unwrap().to_owned();
                            match &*game_selected {
                                "three_kingdoms" => unsafe { app_ui.three_kingdoms.as_mut().unwrap().trigger(); },
                                "warhammer_2" | _ => unsafe { app_ui.warhammer_2.as_mut().unwrap().trigger(); },
                            }
                        }
                    },

                    // PFH4 is for Thrones of Britannia/Warhammer 1/Attila/Rome 2.
                    PFHVersion::PFH4 => {

                        // If we have Warhammer selected, we keep Warhammer. If we have Attila, we keep Attila. That's the logic.
                        let game_selected = GAME_SELECTED.lock().unwrap().to_owned();
                        match &*game_selected {
                            "warhammer" => unsafe { app_ui.warhammer.as_mut().unwrap().trigger(); },
                            "thrones_of_britannia" => unsafe { app_ui.thrones_of_britannia.as_mut().unwrap().trigger(); }
                            "attila" => unsafe { app_ui.attila.as_mut().unwrap().trigger(); }
                            "rome_2" | _ => unsafe { app_ui.rome_2.as_mut().unwrap().trigger(); }
                        }
                    },

                    // PFH3 is for Shogun 2.
                    PFHVersion::PFH3 => unsafe { app_ui.shogun_2.as_mut().unwrap().trigger(); }

                    // PFH0 is for Napoleon/Empire.
                    PFHVersion::PFH0 => {
                        let game_selected = GAME_SELECTED.lock().unwrap().to_owned();
                        match &*game_selected {
                            "napoleon" => unsafe { app_ui.napoleon.as_mut().unwrap().trigger(); },
                            "empire" | _ => unsafe { app_ui.empire.as_mut().unwrap().trigger(); }
                        }
                    },
                }

                // Set the current "Operational Mode" to `Normal`.
                set_my_mod_mode(&mymod_stuff, mode, None);
            }

            // Re-enable the Main Window.
            unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }

            // Destroy whatever it's in the PackedFile's view, to avoid data corruption.
            purge_them_all(&app_ui, packedfiles_open_in_packedfile_view);

            // Close the Global Search stuff and reset the filter's history.
            unsafe { close_global_search_action.as_mut().unwrap().trigger(); }
            if !SETTINGS.lock().unwrap().settings_bool["remember_table_state_permanently"] { TABLE_STATES_UI.lock().unwrap().clear(); }

            // Show the "Tips".
            display_help_tips(&app_ui);

            // Clean the TableStateData.
            *table_state_data.borrow_mut() = TableStateData::new(); 
        }

        // If we got an error...
        Data::Error(error) => {

            // We must check what kind of error it's.
            match error.kind() {

                // If it's the "Generic" error, re-enable the main window and return it.
                ErrorKind::OpenPackFileGeneric(_) => {
                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                    return Err(error)
                }

                // In ANY other situation, it's a message problem.
                _ => panic!(THREADS_MESSAGE_ERROR)
            }
        }

        // In ANY other situation, it's a message problem.
        _ => panic!(THREADS_MESSAGE_ERROR),
    }

    // Return success.
    Ok(())
}

/// This function is used to open ANY supported PackedFile in the right view.
pub fn open_packedfile(
    sender_qt: &Sender<Commands>,
    sender_qt_data: &Sender<Data>,
    receiver_qt: &Rc<RefCell<Receiver<Data>>>,
    app_ui: &AppUI,
    packedfiles_open_in_packedfile_view: &Rc<RefCell<BTreeMap<i32, Rc<RefCell<Vec<String>>>>>>,
    global_search_explicit_paths: &Rc<RefCell<Vec<Vec<String>>>>,
    slots: &Rc<RefCell<Vec<TheOneSlot>>>,
    update_global_search_stuff: *mut Action,
    table_state_data: &Rc<RefCell<BTreeMap<Vec<String>, TableStateData>>>,
    view_position: i32,
) -> Result<()> {

    // Before anything else, we need to check if the TreeView is unlocked. Otherwise we don't do anything from here.
    let allow_opening_packedfile = !(*IS_FOLDER_TREE_VIEW_LOCKED.lock().unwrap());
    if allow_opening_packedfile {
        let selected_items = get_item_types_from_main_treeview_selection(app_ui);
        let item_type = if selected_items.len() == 1 { &selected_items[0] } else { return Ok(()) };
        match item_type {

            // Only in case it's a file, we do something.
            TreePathType::File(path) => {

                // If the file we want to open is already open in another view, don't open it.
                for (view_pos, packed_file_path) in packedfiles_open_in_packedfile_view.borrow().iter() {
                    if &*packed_file_path.borrow() == path && view_pos != &view_position {
                        return Err(ErrorKind::PackedFileIsOpenInAnotherView)?
                    }
                }

                // We get his type to decode it properly
                let packed_file_type = get_packed_file_type(&path);

                // Create the widget that'll act as a container for the view.
                let widget = Widget::new().into_raw();
                let widget_layout = create_grid_layout_unsafe(widget);

                // Put the Path into a Rc<RefCell<> so we can alter it while it's open.
                let path = Rc::new(RefCell::new(path.to_vec()));

                // Then, depending of his type we decode it properly (if we have it implemented support
                // for his type).
                match packed_file_type {

                    // If the file is a Loc PackedFile...
                    DecodeablePackedFileType::Loc => {

                        // Try to get the view build, or return error.
                        match create_loc_view(
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt,
                            &app_ui,
                            widget_layout,
                            &path,
                            &global_search_explicit_paths,
                            update_global_search_stuff,
                            table_state_data,
                        ) {
                            Ok(new_slots) => { slots.borrow_mut().push(TheOneSlot::Table(new_slots)); },
                            Err(error) => return Err(ErrorKind::LocDecode(format!("{}", error)))?,
                        }

                        // Tell the program there is an open PackedFile and finish the table.
                        purge_that_one_specifically(&app_ui, view_position, &packedfiles_open_in_packedfile_view);
                        packedfiles_open_in_packedfile_view.borrow_mut().insert(view_position, path);
                        unsafe { app_ui.packed_file_splitter.as_mut().unwrap().insert_widget(view_position, widget as *mut Widget); }
                    }

                    // If the file is a DB PackedFile...
                    DecodeablePackedFileType::DB => {

                        // Try to get the view build, or return error.
                        match create_db_view(
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt,
                            &app_ui,
                            widget_layout,
                            &path,
                            &global_search_explicit_paths,
                            update_global_search_stuff,
                            table_state_data
                        ) {
                            Ok(new_slots) => { slots.borrow_mut().push(TheOneSlot::Table(new_slots)); },
                            Err(error) => return Err(ErrorKind::DBTableDecode(format!("{}", error)))?,
                        }

                        // Tell the program there is an open PackedFile and finish the table.
                        purge_that_one_specifically(&app_ui, view_position, &packedfiles_open_in_packedfile_view);
                        packedfiles_open_in_packedfile_view.borrow_mut().insert(view_position, path);
                        unsafe { app_ui.packed_file_splitter.as_mut().unwrap().insert_widget(view_position, widget as *mut Widget); }

                        // Disable the "Change game selected" function, so we cannot change the current schema with an open table.
                        unsafe { app_ui.game_selected_group.as_mut().unwrap().set_enabled(false); }
                    }

                    // If the file is a Text PackedFile...
                    DecodeablePackedFileType::Text => {
                        
                        // Try to get the view build, or return error.
                        match create_text_view(
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt,
                            &app_ui,
                            widget_layout,
                            &path,
                            &packedfiles_open_in_packedfile_view
                        ) {
                            Ok(new_slots) => { slots.borrow_mut().push(TheOneSlot::Text(new_slots)); },
                            Err(error) => return Err(ErrorKind::TextDecode(format!("{}", error)))?,
                        }

                        // Tell the program there is an open PackedFile and finish the table.
                        purge_that_one_specifically(&app_ui, view_position, &packedfiles_open_in_packedfile_view);
                        packedfiles_open_in_packedfile_view.borrow_mut().insert(view_position, path);
                        unsafe { app_ui.packed_file_splitter.as_mut().unwrap().insert_widget(view_position, widget as *mut Widget); }
                    }

                    // If the file is a Text PackedFile...
                    DecodeablePackedFileType::RigidModel => {

                        // Try to get the view build, or return error.
                        match PackedFileRigidModelDataView::create_data_view(
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt,
                            &app_ui,
                            widget_layout,
                            &path
                        ) {
                            Ok(new_slots) => { slots.borrow_mut().push(TheOneSlot::RigidModel(new_slots)); },
                            Err(error) => return Err(ErrorKind::RigidModelDecode(format!("{}", error)))?,
                        }

                        // Tell the program there is an open PackedFile and finish the table.
                        purge_that_one_specifically(&app_ui, view_position, &packedfiles_open_in_packedfile_view);
                        packedfiles_open_in_packedfile_view.borrow_mut().insert(view_position, path);
                        unsafe { app_ui.packed_file_splitter.as_mut().unwrap().insert_widget(view_position, widget as *mut Widget); }
                    }

                    // If the file is a Text PackedFile...
                    DecodeablePackedFileType::Image => {

                        // Try to get the view build, or return error.
                        if let Err(error) = ui::packedfile_image::create_image_view(
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt,
                            widget_layout,
                            &path,
                        ) { return Err(ErrorKind::ImageDecode(format!("{}", error)))? }

                        // Tell the program there is an open PackedFile and finish the table.
                        purge_that_one_specifically(&app_ui, view_position, &packedfiles_open_in_packedfile_view);
                        packedfiles_open_in_packedfile_view.borrow_mut().insert(view_position, path);
                        unsafe { app_ui.packed_file_splitter.as_mut().unwrap().insert_widget(view_position, widget as *mut Widget); }
                    }

                    // For any other PackedFile, just restore the display tips.
                    _ => {
                        purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);
                        display_help_tips(&app_ui);
                    }
                }
            }

            // If it's anything else, then we just show the "Tips" list.
            _ => {
                purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);
                display_help_tips(&app_ui);
            }
        }
    }

    Ok(())
}

/// This function is used to save ANY supported PackFile. If the PackFile doesn't exist or we want to save it
/// with another name, it opens a dialog asking for a path.
pub fn save_packfile(
    is_as_other_file: bool,
    app_ui: &AppUI,
    mode: &Rc<RefCell<Mode>>,
    mymod_stuff: &Rc<RefCell<MyModStuff>>,
    sender_qt: &Sender<Commands>,
    sender_qt_data: &Sender<Data>,
    receiver_qt: &Rc<RefCell<Receiver<Data>>>,
    table_state_data: &Rc<RefCell<BTreeMap<Vec<String>, TableStateData>>>,
    packedfiles_open_in_packedfile_view: &Rc<RefCell<BTreeMap<i32, Rc<RefCell<Vec<String>>>>>>,
) -> Result<()> {

    // If we are saving with the "Save PackFile" button, we try to save it. If we detect the PackFile doesn't exist,
    // we fall back to the "Save PackFile As" behavior, asking the user for a Path.
    let mut result = Ok(());
    let mut do_we_need_to_save_as = false;
    if !is_as_other_file {
        unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
        sender_qt.send(Commands::SavePackFile).unwrap();

        match check_message_validity_tryrecv(&receiver_qt) {
            Data::I64(date) => {

                // Clean the TreeView and reset the 'Last Modified Date' of the PackFile.
                update_treeview(
                    &sender_qt,
                    &sender_qt_data,
                    &receiver_qt,
                    &app_ui,
                    app_ui.folder_tree_view,
                    Some(app_ui.folder_tree_filter),
                    app_ui.folder_tree_model,
                    TreeViewOperation::Clean,
                );
                unsafe { app_ui.folder_tree_model.as_mut().unwrap().item(0).as_mut().unwrap().set_tool_tip(&QString::from_std_str(format!("Last Modified: {:?}", NaiveDateTime::from_timestamp(date, 0)))); }
            }

            Data::Error(error) => {
                match error.kind() {
                    ErrorKind::PackFileIsNotAFile => do_we_need_to_save_as = true,

                    // If there was any other error while saving the PackFile, report it. Any other error should trigger a Panic.
                    ErrorKind::SavePackFileGeneric(_) => result = Err(error),
                    _ => panic!(THREADS_MESSAGE_ERROR)
                }
            }
            _ => panic!(THREADS_MESSAGE_ERROR)
        }
    }

    // If we want instead to save as, or the normal save has default to this, we try to save the PackFile as another
    // Packfile, asking for a path first.
    if is_as_other_file || do_we_need_to_save_as {
        unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
        sender_qt.send(Commands::SavePackFileAs).unwrap();
        match check_message_validity_recv2(&receiver_qt) {
            Data::PathBuf(file_path) => {

                // Create the FileDialog to save the PackFile and configure it.
                let mut file_dialog = unsafe { FileDialog::new_unsafe((
                    app_ui.window as *mut Widget,
                    &QString::from_std_str("Save PackFile"),
                )) };
                file_dialog.set_accept_mode(qt_widgets::file_dialog::AcceptMode::Save);
                file_dialog.set_name_filter(&QString::from_std_str("PackFiles (*.pack)"));
                file_dialog.set_confirm_overwrite(true);
                file_dialog.set_default_suffix(&QString::from_std_str("pack"));
                file_dialog.select_file(&QString::from_std_str(&file_path.file_name().unwrap().to_string_lossy()));

                // If we are saving an existing PackFile with another name, we start in his current path.
                if file_path.is_file() {
                    let mut path = file_path.to_path_buf();
                    path.pop();
                    file_dialog.set_directory(&QString::from_std_str(path.to_string_lossy().as_ref().to_owned()));
                }

                // In case we have a default path for the Game Selected and that path is valid,
                // we use his data folder as base path for saving our PackFile.
                else if let Some(ref path) = get_game_selected_data_path() {
                    if path.is_dir() { file_dialog.set_directory(&QString::from_std_str(path.to_string_lossy().as_ref().to_owned())); }
                }

                // Run it and act depending on the response we get (1 => Accept, 0 => Cancel).
                if file_dialog.exec() == 1 {
                    let path = PathBuf::from(file_dialog.selected_files().at(0).to_std_string());
                    sender_qt_data.send(Data::PathBuf(path.to_path_buf())).unwrap();

                    // Check what happened when we tried to save the PackFile.
                    match check_message_validity_tryrecv(&receiver_qt) {
                        Data::I64(date) => {

                            // Update the "Last Modified Date" of the PackFile in the TreeView and his name.
                            unsafe { app_ui.folder_tree_model.as_mut().unwrap().item(0).as_mut().unwrap().set_tool_tip(&QString::from_std_str(format!("Last Modified: {:?}", NaiveDateTime::from_timestamp(date, 0)))); }
                            unsafe { app_ui.folder_tree_model.as_mut().unwrap().item(0).as_mut().unwrap().set_text(&QString::from_std_str(path.file_name().unwrap().to_string_lossy().as_ref().to_owned())); }

                            // Clean the TreeView.
                            update_treeview(
                                &sender_qt,
                                &sender_qt_data,
                                &receiver_qt,
                                &app_ui,
                                app_ui.folder_tree_view,
                                Some(app_ui.folder_tree_filter),
                                app_ui.folder_tree_model,
                                TreeViewOperation::Clean,
                            );

                            // Set the current "Operational Mode" to Normal, as this is a "New" mod.
                            set_my_mod_mode(&mymod_stuff, &mode, None);
                        }

                        // If it's an error we can dealt with, report it.
                        Data::Error(error) => {
                            match error.kind() {
                                ErrorKind::SavePackFileGeneric(_) => result = Err(error),
                                _ => panic!(THREADS_MESSAGE_ERROR),
                            }
                        }
                        _ => panic!(THREADS_MESSAGE_ERROR)
                    }
                }

                // Otherwise, we take it as we canceled the save in some way, so we tell the Background Loop to stop waiting.
                else { sender_qt_data.send(Data::Cancel).unwrap(); }
            }

            // If there was an error report it, if we can.
            Data::Error(error) => {
                match error.kind() {
                    ErrorKind::PackFileIsNonEditable => result = Err(error),
                    _ => panic!(THREADS_MESSAGE_ERROR)
                }
            }
            _ => panic!(THREADS_MESSAGE_ERROR)
        }
    }

    // Then we re-enable the main Window and return whatever we've received.
    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }

    // Clean all the modified items EXCEPT those open. That way we can still undo changes there.
    if result.is_ok() { 
        let iter = table_state_data.borrow().iter().map(|x| x.0).cloned().collect::<Vec<Vec<String>>>();
        for path in &iter {
            if !packedfiles_open_in_packedfile_view.borrow().values().any(|x| *x.borrow() == *path) {
                table_state_data.borrow_mut().remove(path);
            }
        }
    }
    result
}

/// This function takes care of the re-creation of the "MyMod" list in the following moments:
/// - At the start of the program.
/// - At the end of MyMod deletion.
/// - At the end of MyMod creation.
/// - At the end of settings update.
/// We need to return a tuple with the actions (for further manipulation) and the slots (to keep them alive).
pub fn build_my_mod_menu(
    sender_qt: &Sender<Commands>,
    sender_qt_data: &Sender<Data>,
    receiver_qt: &Rc<RefCell<Receiver<Data>>>,
    app_ui: AppUI,
    menu_bar_mymod: *mut Menu,
    mode: &Rc<RefCell<Mode>>,
    needs_rebuild: Rc<RefCell<bool>>,
    packedfiles_open_in_packedfile_view: &Rc<RefCell<BTreeMap<i32, Rc<RefCell<Vec<String>>>>>>,
    close_global_search_action: *mut Action,
    table_state_data: &Rc<RefCell<BTreeMap<Vec<String>, TableStateData>>>,
) -> (MyModStuff, MyModSlots) {

    //---------------------------------------------------------------------------------------//
    // Build the "Static" part of the menu...
    //---------------------------------------------------------------------------------------//

    // First, we clear the list, just in case this is a "Rebuild" of the menu.
    unsafe { menu_bar_mymod.as_mut().unwrap().clear(); }

    // Then, we create the actions again.
    let mymod_stuff = unsafe { MyModStuff {
            new_mymod: menu_bar_mymod.as_mut().unwrap().add_action(&QString::from_std_str("&New MyMod")),
            delete_selected_mymod: menu_bar_mymod.as_mut().unwrap().add_action(&QString::from_std_str("&Delete Selected MyMod")),
            install_mymod: menu_bar_mymod.as_mut().unwrap().add_action(&QString::from_std_str("&Install")),
            uninstall_mymod: menu_bar_mymod.as_mut().unwrap().add_action(&QString::from_std_str("&Uninstall")),
        }
    };

    // Add a separator in the middle of the menu.
    unsafe { menu_bar_mymod.as_mut().unwrap().insert_separator(mymod_stuff.install_mymod); }

    // And we create the slots.
    let mut mymod_slots = MyModSlots {

        // This slot is used for the "New MyMod" action.
        new_mymod: SlotBool::new(clone!(
            sender_qt,
            sender_qt_data,
            receiver_qt,
            packedfiles_open_in_packedfile_view,
            table_state_data,
            app_ui,
            mode,
            needs_rebuild => move |_| {

                // Create the "New MyMod" Dialog, and get the result.
                match NewMyModDialog::create_new_mymod_dialog(&app_ui) {

                    // If we accepted...
                    Some(data) => {

                        // Get the info about the new MyMod.
                        let mod_name = data.0;
                        let mod_game = data.1;

                        // Get the PackFile's name.
                        let full_mod_name = format!("{}.pack", mod_name);

                        // Change the Game Selected to match the one we chose for the new "MyMod".
                        // NOTE: Arena should not be on this list.
                        match &*mod_game {
                            "three_kingdoms" => unsafe { app_ui.three_kingdoms.as_mut().unwrap().trigger(); }
                            "warhammer_2" => unsafe { app_ui.warhammer_2.as_mut().unwrap().trigger(); }
                            "warhammer" => unsafe { app_ui.warhammer.as_mut().unwrap().trigger(); }
                            "thrones_of_britannia" => unsafe { app_ui.thrones_of_britannia.as_mut().unwrap().trigger(); }
                            "attila" => unsafe { app_ui.attila.as_mut().unwrap().trigger(); }
                            "rome_2" => unsafe { app_ui.rome_2.as_mut().unwrap().trigger(); }
                            "shogun_2" => unsafe { app_ui.shogun_2.as_mut().unwrap().trigger(); }
                            "napoleon" => unsafe { app_ui.napoleon.as_mut().unwrap().trigger(); }
                            "empire" | _ => unsafe { app_ui.empire.as_mut().unwrap().trigger(); }
                        }

                        // Get his new path from the base "MyMod" path + `mod_game`.
                        let mut mymod_path = SETTINGS.lock().unwrap().paths["mymods_base_path"].clone().unwrap();
                        mymod_path.push(&mod_game);

                        // Just in case the folder doesn't exist, we try to create it.
                        if DirBuilder::new().recursive(true).create(&mymod_path).is_err() {
                            return show_dialog(app_ui.window, false, ErrorKind::IOCreateAssetFolder);
                        }

                        // We need to create another folder inside the game's folder with the name of the new "MyMod", to store extracted files.
                        let mut mymod_path_private = mymod_path.to_path_buf();
                        mymod_path_private.push(&mod_name);
                        if DirBuilder::new().recursive(true).create(&mymod_path_private).is_err() {
                            return show_dialog(app_ui.window, false, ErrorKind::IOCreateNestedAssetFolder);
                        };

                        // Add the PackFile's name to the full path.
                        mymod_path.push(&full_mod_name);

                        // Tell the Background Thread to create a new PackFile.
                        sender_qt.send(Commands::NewPackFile).unwrap();
                        let _ = if let Data::U32(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // Tell the Background Thread to create a new PackFile.
                        sender_qt.send(Commands::SavePackFileAs).unwrap();
                        let _ = if let Data::PathBuf(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // Pass the new PackFile's Path to the worker thread.
                        sender_qt_data.send(Data::PathBuf(mymod_path.to_path_buf())).unwrap();

                        // Check what response we got.
                        match check_message_validity_tryrecv(&receiver_qt) {
                        
                            // If it's success....
                            Data::I64(_) => {

                                // Destroy whatever it's in the PackedFile's view, to avoid data corruption.
                                purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);

                                // Close the Global Search stuff and reset the filter's history.
                                unsafe { close_global_search_action.as_mut().unwrap().trigger(); }
                                if !SETTINGS.lock().unwrap().settings_bool["remember_table_state_permanently"] { TABLE_STATES_UI.lock().unwrap().clear(); }

                                // Show the "Tips".
                                display_help_tips(&app_ui);

                                // Update the TreeView.
                                update_treeview(
                                    &sender_qt,
                                    &sender_qt_data,
                                    &receiver_qt,
                                    &app_ui,
                                    app_ui.folder_tree_view,
                                    Some(app_ui.folder_tree_filter),
                                    app_ui.folder_tree_model,
                                    TreeViewOperation::Build(false),
                                );

                                // Mark it as "Mod" in the UI.
                                unsafe { app_ui.change_packfile_type_mod.as_mut().unwrap().set_checked(true); }

                                // By default, the four bitmask should be false.
                                unsafe { app_ui.change_packfile_type_data_is_encrypted.as_mut().unwrap().set_checked(false); }
                                unsafe { app_ui.change_packfile_type_index_includes_timestamp.as_mut().unwrap().set_checked(false); }
                                unsafe { app_ui.change_packfile_type_index_is_encrypted.as_mut().unwrap().set_checked(false); }
                                unsafe { app_ui.change_packfile_type_header_is_extended.as_mut().unwrap().set_checked(false); }

                                // Enable the actions available for the PackFile from the `MenuBar`.
                                enable_packfile_actions(&app_ui, &Rc::new(RefCell::new(mymod_stuff)), true);

                                // Set the current "Operational Mode" to `MyMod`.
                                set_my_mod_mode(&Rc::new(RefCell::new(mymod_stuff)), &mode, Some(mymod_path));

                                // Set it to rebuild next time we try to open the "MyMod" Menu.
                                *needs_rebuild.borrow_mut() = true;

                                // Clean the TableStateData.
                                *table_state_data.borrow_mut() = TableStateData::new(); 
                            },

                            // If we got an error...
                            Data::Error(error) => {

                                // We must check what kind of error it's.
                                match error.kind() {

                                    // If there was any other error while saving the PackFile, report it and break the loop.
                                    ErrorKind::SavePackFileGeneric(_) => show_dialog(app_ui.window, false, error),

                                    // In ANY other situation, it's a message problem.
                                    _ => panic!(THREADS_MESSAGE_ERROR)
                                }
                            }

                            // In ANY other situation, it's a message problem.
                            _ => panic!(THREADS_MESSAGE_ERROR),
                        }
                    }

                    // If we canceled the creation of a "MyMod", just return.
                    None => return,
                }
            }
        )),

        // This slot is used for the "Delete Selected MyMod" action.
        delete_selected_mymod: SlotBool::new(clone!(
            sender_qt,
            sender_qt_data,
            receiver_qt,
            mode,
            app_ui => move |_| {

                // Ask before doing it, as this will permanently delete the mod from the Disk.
                if are_you_sure(&app_ui, true) {

                    // We want to keep our "MyMod" name for the success message, so we store it here.
                    let old_mod_name: String;

                    // Try to delete the "MyMod" and his folder.
                    let mod_deleted = match *mode.borrow() {

                        // If we have a "MyMod" selected...
                        Mode::MyMod {ref game_folder_name, ref mod_name} => {

                            // We save the name of the PackFile for later use.
                            old_mod_name = mod_name.to_owned();

                            // And the "MyMod" path is configured...
                            if let Some(ref mymods_base_path) = SETTINGS.lock().unwrap().paths["mymods_base_path"] {

                                // We get his path.
                                let mut mymod_path = mymods_base_path.to_path_buf();
                                mymod_path.push(&game_folder_name);
                                mymod_path.push(&mod_name);

                                // If the mod doesn't exist, return error.
                                if !mymod_path.is_file() { return show_dialog(app_ui.window, false, ErrorKind::MyModPackFileDoesntExist); }

                                // And we try to delete his PackFile. If it fails, return error.
                                if remove_file(&mymod_path).is_err() {
                                    return show_dialog(app_ui.window, false, ErrorKind::IOGenericDelete(vec![mymod_path; 1]));
                                }

                                // Now we get his assets folder.
                                let mut mymod_assets_path = mymod_path.to_path_buf();
                                mymod_assets_path.pop();
                                mymod_assets_path.push(&mymod_path.file_stem().unwrap().to_string_lossy().as_ref().to_owned());

                                // We check that path exists. This is optional, so it should allow the deletion
                                // process to continue with a warning.
                                if !mymod_assets_path.is_dir() {
                                    show_dialog(app_ui.window, false, ErrorKind::MyModPackFileDeletedFolderNotFound);
                                }

                                // If the assets folder exists, we try to delete it. Again, this is optional, so it should not stop the deleting process.
                                else if remove_dir_all(&mymod_assets_path).is_err() {
                                    show_dialog(app_ui.window, false, ErrorKind::IOGenericDelete(vec![mymod_assets_path; 1]));
                                }

                                // We return true, as we have delete the files of the "MyMod".
                                true
                            }

                            // If the "MyMod" path is not configured, return an error.
                            else { return show_dialog(app_ui.window, false, ErrorKind::MyModPathNotConfigured); }
                        }

                        // If we don't have a "MyMod" selected, return an error.
                        Mode::Normal => return show_dialog(app_ui.window, false, ErrorKind::MyModDeleteWithoutMyModSelected),
                    };

                    // If we deleted the "MyMod", we allow chaos to form below.
                    if mod_deleted {

                        // Set the current "Operational Mode" to `Normal`.
                        set_my_mod_mode(&Rc::new(RefCell::new(mymod_stuff)), &mode, None);

                        // Create a "dummy" PackFile, effectively closing the currently open PackFile.
                        sender_qt.send(Commands::ResetPackFile).unwrap();

                        // Disable the actions available for the PackFile from the `MenuBar`.
                        enable_packfile_actions(&app_ui, &Rc::new(RefCell::new(mymod_stuff)), false);

                        // Set the dummy mod as "Not modified".
                        update_treeview(
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt,
                            &app_ui,
                            app_ui.folder_tree_view,
                            Some(app_ui.folder_tree_filter),
                            app_ui.folder_tree_model,
                            TreeViewOperation::Clear,
                        );

                        // Set it to rebuild next time we try to open the MyMod Menu.
                        *needs_rebuild.borrow_mut() = true;

                        // Show the "MyMod" deleted Dialog.
                        show_dialog(app_ui.window, true, format!("MyMod successfully deleted: \"{}\".", old_mod_name));
                    }
                }
            }
        )),

        // This slot is used for the "Install MyMod" action.
        install_mymod: SlotBool::new(clone!(
            mode,
            app_ui => move |_| {

                // Depending on our current "Mode", we choose what to do.
                match *mode.borrow() {

                    // If we have a "MyMod" selected...
                    Mode::MyMod {ref game_folder_name, ref mod_name} => {

                        // And the "MyMod" path is configured...
                        let settings = SETTINGS.lock().unwrap().clone();
                        let mymods_base_path = &settings.paths["mymods_base_path"];
                        if let Some(ref mymods_base_path) = mymods_base_path {

                            // If we have a `game_data_path` for the current `GameSelected`...
                            if let Some(mut game_data_path) = get_game_selected_data_path() {

                                // We get the "MyMod"s PackFile path.
                                let mut mymod_path = mymods_base_path.to_path_buf();
                                mymod_path.push(&game_folder_name);
                                mymod_path.push(&mod_name);

                                // We check that the "MyMod"s PackFile exists.
                                if !mymod_path.is_file() { return show_dialog(app_ui.window, false, ErrorKind::MyModPackFileDoesntExist); }

                                // We check that the destination path exists.
                                if !game_data_path.is_dir() {
                                    return show_dialog(app_ui.window, false, ErrorKind::MyModInstallFolderDoesntExists);
                                }

                                // Get the destination path for the PackFile with the PackFile name included.
                                game_data_path.push(&mod_name);

                                // And copy the PackFile to his destination. If the copy fails, return an error.
                                if copy(mymod_path, game_data_path.to_path_buf()).is_err() {
                                    return show_dialog(app_ui.window, false, ErrorKind::IOGenericCopy(game_data_path));
                                }
                            }

                            // If we don't have a `game_data_path` configured for the current `GameSelected`...
                            else { return show_dialog(app_ui.window, false, ErrorKind::GamePathNotConfigured); }
                        }

                        // If the "MyMod" path is not configured, return an error.
                        else { show_dialog(app_ui.window, false, ErrorKind::MyModPathNotConfigured); }
                    }

                    // If we have no "MyMod" selected, return an error.
                    Mode::Normal => show_dialog(app_ui.window, false, ErrorKind::MyModDeleteWithoutMyModSelected),
                }

            }
        )),

        // This slot is used for the "Uninstall MyMod" action.
        uninstall_mymod: SlotBool::new(clone!(
            mode,
            app_ui => move |_| {

                // Depending on our current "Mode", we choose what to do.
                match *mode.borrow() {

                    // If we have a "MyMod" selected...
                    Mode::MyMod {ref mod_name,..} => {

                        // If we have a `game_data_path` for the current `GameSelected`...
                        if let Some(mut game_data_path) = get_game_selected_data_path() {

                            // Get the destination path for the PackFile with the PackFile included.
                            game_data_path.push(&mod_name);

                            // We check that the "MyMod" is actually installed in the provided path.
                            if !game_data_path.is_file() { return show_dialog(app_ui.window, false, ErrorKind::MyModNotInstalled); }

                            // If the "MyMod" is installed, we remove it. If there is a problem deleting it, return an error dialog.
                            else if remove_file(game_data_path.to_path_buf()).is_err() {
                                return show_dialog(app_ui.window, false, ErrorKind::IOGenericDelete(vec![game_data_path; 1]));
                            }
                        }

                        // If we don't have a `game_data_path` configured for the current `GameSelected`...
                        else { show_dialog(app_ui.window, false, ErrorKind::GamePathNotConfigured); }
                    }

                    // If we have no MyMod selected, return an error.
                    Mode::Normal => show_dialog(app_ui.window, false, ErrorKind::MyModDeleteWithoutMyModSelected),
                }
            }
        )),

        // This is an empty list to populate later with the slots used to open every "MyMod" we have.
        open_mymod: vec![],
    };

    // "About" Menu Actions.
    unsafe { mymod_stuff.new_mymod.as_ref().unwrap().signals().triggered().connect(&mymod_slots.new_mymod); }
    unsafe { mymod_stuff.delete_selected_mymod.as_ref().unwrap().signals().triggered().connect(&mymod_slots.delete_selected_mymod); }
    unsafe { mymod_stuff.install_mymod.as_ref().unwrap().signals().triggered().connect(&mymod_slots.install_mymod); }
    unsafe { mymod_stuff.uninstall_mymod.as_ref().unwrap().signals().triggered().connect(&mymod_slots.uninstall_mymod); }

    // Status bar tips.
    unsafe { mymod_stuff.new_mymod.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the dialog to create a new MyMod.")); }
    unsafe { mymod_stuff.delete_selected_mymod.as_mut().unwrap().set_status_tip(&QString::from_std_str("Delete the currently selected MyMod.")); }
    unsafe { mymod_stuff.install_mymod.as_mut().unwrap().set_status_tip(&QString::from_std_str("Copy the currently selected MyMod into the data folder of the GameSelected.")); }
    unsafe { mymod_stuff.uninstall_mymod.as_mut().unwrap().set_status_tip(&QString::from_std_str("Removes the currently selected MyMod from the data folder of the GameSelected.")); }

    //---------------------------------------------------------------------------------------//
    // Build the "Dynamic" part of the menu...
    //---------------------------------------------------------------------------------------//

    // Add a separator for this section.
    unsafe { menu_bar_mymod.as_mut().unwrap().add_separator(); }

    // If we have the "MyMod" path configured...
    if let Some(ref mymod_base_path) = SETTINGS.lock().unwrap().paths["mymods_base_path"] {

        // And can get without errors the folders in that path...
        if let Ok(game_folder_list) = mymod_base_path.read_dir() {

            // We get all the games that have mods created (Folder exists and has at least a *.pack file inside).
            for game_folder in game_folder_list {

                // If the file/folder is valid...
                if let Ok(game_folder) = game_folder {

                    // Get the list of supported games folders.
                    let supported_folders = SUPPORTED_GAMES.iter().filter(|(_, x)| x.supports_editing).map(|(folder_name,_)| folder_name.to_string()).collect::<Vec<String>>();

                    // If it's a valid folder, and it's in our supported games list...
                    if game_folder.path().is_dir() && supported_folders.contains(&game_folder.file_name().to_string_lossy().as_ref().to_owned()) {

                        // We create that game's menu here.
                        let game_folder_name = game_folder.file_name().to_string_lossy().as_ref().to_owned();
                        let game_display_name = &SUPPORTED_GAMES[&*game_folder_name].display_name;

                        let mut game_submenu = Menu::new(&QString::from_std_str(&game_display_name));

                        // If there were no errors while reading the path...
                        if let Ok(game_folder_files) = game_folder.path().read_dir() {

                            // We need to sort these files, so they appear sorted in the menu.
                            let mut game_folder_files_sorted: Vec<_> = game_folder_files.map(|x| x.unwrap().path()).collect();
                            game_folder_files_sorted.sort();

                            // We get all the stuff in that game's folder...
                            for pack_file in &game_folder_files_sorted {

                                // And it's a file that ends in .pack...
                                if pack_file.is_file() && pack_file.extension().unwrap_or_else(||OsStr::new("invalid")).to_string_lossy() == "pack" {

                                    // That means our file is a valid PackFile and it needs to be added to the menu.
                                    let mod_name = pack_file.file_name().unwrap().to_string_lossy().as_ref().to_owned();

                                    // Create the action for it.
                                    let open_mod_action = game_submenu.add_action(&QString::from_std_str(mod_name));

                                    // Get this into an Rc so we can pass it to the "Open PackFile" closure.
                                    let mymod_stuff = Rc::new(RefCell::new(mymod_stuff));

                                    // Create the slot for that action.
                                    let slot_open_mod = SlotBool::new(clone!(
                                        game_folder_name,
                                        mode,
                                        mymod_stuff,
                                        pack_file,
                                        packedfiles_open_in_packedfile_view,
                                        close_global_search_action,
                                        table_state_data,
                                        sender_qt,
                                        sender_qt_data,
                                        receiver_qt => move |_| {

                                            // Check first if there has been changes in the PackFile.
                                            if are_you_sure(&app_ui, false) {

                                                // Open the PackFile (or die trying it!).
                                                if let Err(error) = open_packfile(
                                                    &sender_qt,
                                                    &sender_qt_data,
                                                    &receiver_qt,
                                                    &[pack_file.to_path_buf()],
                                                    &app_ui,
                                                    &mymod_stuff,
                                                    &mode,
                                                    &game_folder_name,
                                                    &packedfiles_open_in_packedfile_view,
                                                    close_global_search_action,
                                                    &table_state_data,
                                                ) { show_dialog(app_ui.window, false, error) }
                                            }
                                        }
                                    ));

                                    // Add the slot to the list.
                                    mymod_slots.open_mymod.push(slot_open_mod);

                                    // Connect the action to the slot.
                                    unsafe { open_mod_action.as_ref().unwrap().signals().triggered().connect(mymod_slots.open_mymod.last().unwrap()); }
                                }
                            }
                        }

                        // Only if the submenu has items, we add it to the big menu.
                        if game_submenu.actions().count() > 0 {
                            unsafe { menu_bar_mymod.as_mut().unwrap().add_menu_unsafe(game_submenu.into_raw()); }
                        }
                    }
                }
            }
        }
    }

    // If there is a "MyMod" path set in the settings...
    if let Some(ref path) = SETTINGS.lock().unwrap().paths["mymods_base_path"] {

        // And it's a valid directory, enable the "New MyMod" button.
        if path.is_dir() { unsafe { mymod_stuff.new_mymod.as_mut().unwrap().set_enabled(true); }}

        // Otherwise, disable it.
        else { unsafe { mymod_stuff.new_mymod.as_mut().unwrap().set_enabled(false); }}
    }

    // Otherwise, disable it.
    else { unsafe { mymod_stuff.new_mymod.as_mut().unwrap().set_enabled(false); }}

    // If we just created a MyMod, these options should be enabled.
    if let Mode::MyMod{..} = *mode.borrow() {
        unsafe { mymod_stuff.delete_selected_mymod.as_mut().unwrap().set_enabled(true); }
        unsafe { mymod_stuff.install_mymod.as_mut().unwrap().set_enabled(true); }
        unsafe { mymod_stuff.uninstall_mymod.as_mut().unwrap().set_enabled(true); }
    }

    // Otherwise, disable by default the rest of the actions.
    else {   
        unsafe { mymod_stuff.delete_selected_mymod.as_mut().unwrap().set_enabled(false); }
        unsafe { mymod_stuff.install_mymod.as_mut().unwrap().set_enabled(false); }
        unsafe { mymod_stuff.uninstall_mymod.as_mut().unwrap().set_enabled(false); }
    }

    // Return the MyModStuff with all the new actions.
    (mymod_stuff, mymod_slots)
}


/// This function takes care of the re-creation of the "Open From Content" and "Open From Data" submenus.
/// This has to be executed every time we change the Game Selected.
pub fn build_open_from_submenus(
    sender_qt: &Sender<Commands>,
    sender_qt_data: &Sender<Data>,
    receiver_qt: &Rc<RefCell<Receiver<Data>>>,
    app_ui: AppUI,
    submenu_open_from_content: *mut Menu,
    submenu_open_from_data: *mut Menu,
    mode: &Rc<RefCell<Mode>>,
    packedfiles_open_in_packedfile_view: &Rc<RefCell<BTreeMap<i32, Rc<RefCell<Vec<String>>>>>>,
    mymod_stuff: &Rc<RefCell<MyModStuff>>,
    close_global_search_action: *mut Action,
    table_state_data: &Rc<RefCell<BTreeMap<Vec<String>, TableStateData>>>,
) -> Vec<SlotBool<'static>> {

    // First, we clear the list, just in case this is a "Rebuild" of the menu.
    unsafe { submenu_open_from_content.as_mut().unwrap().clear(); }
    unsafe { submenu_open_from_data.as_mut().unwrap().clear(); }

    // And we create the slots.
    let mut open_from_slots = vec![];

    //---------------------------------------------------------------------------------------//
    // Build the menus...
    //---------------------------------------------------------------------------------------//

    // Get the path of every PackFile in the data folder (if it's configured) and make an action for each one of them.
    if let Some(ref mut paths) = get_game_selected_content_packfiles_paths() {
        paths.sort_unstable_by_key(|x| x.file_name().unwrap().to_string_lossy().as_ref().to_owned());
        for path in paths {

            // That means our file is a valid PackFile and it needs to be added to the menu.
            let mod_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();

            // Create the action for it.
            let open_mod_action;
            unsafe { open_mod_action = submenu_open_from_content.as_mut().unwrap().add_action(&QString::from_std_str(mod_name)); }

            // Create the slot for that action.
            let slot_open_mod = SlotBool::new(clone!(
                mode,
                mymod_stuff,
                path,
                packedfiles_open_in_packedfile_view,
                close_global_search_action,
                table_state_data,
                sender_qt,
                sender_qt_data,
                receiver_qt => move |_| {

                    // Check first if there has been changes in the PackFile.
                    if are_you_sure(&app_ui, false) {

                        // Try to open it, and report it case of error.
                        if let Err(error) = open_packfile(
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt,
                            &[path.to_path_buf()],
                            &app_ui,
                            &mymod_stuff,
                            &mode,
                            "",
                            &packedfiles_open_in_packedfile_view,
                            close_global_search_action,
                            &table_state_data,
                        ) { show_dialog(app_ui.window, false, error); }
                    }
                }
            ));

            // Add the slot to the list.
            open_from_slots.push(slot_open_mod);

            // Connect the action to the slot.
            unsafe { open_mod_action.as_ref().unwrap().signals().triggered().connect(open_from_slots.last().unwrap()); }  
        }
    }

    // Get the path of every PackFile in the data folder (if it's configured) and make an action for each one of them.
    if let Some(ref mut paths) = get_game_selected_data_packfiles_paths() {
        paths.sort_unstable_by_key(|x| x.file_name().unwrap().to_string_lossy().as_ref().to_owned());
        for path in paths {

            // That means our file is a valid PackFile and it needs to be added to the menu.
            let mod_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();

            // Create the action for it.
            let open_mod_action;
            unsafe { open_mod_action = submenu_open_from_data.as_mut().unwrap().add_action(&QString::from_std_str(mod_name)); }

            // Create the slot for that action.
            let slot_open_mod = SlotBool::new(clone!(
                mode,
                mymod_stuff,
                path,
                packedfiles_open_in_packedfile_view,
                close_global_search_action,
                table_state_data,
                sender_qt,
                sender_qt_data,
                receiver_qt => move |_| {

                    // Check first if there has been changes in the PackFile.
                    if are_you_sure(&app_ui, false) {

                        // Try to open it, and report it case of error.
                        if let Err(error) = open_packfile(
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt,
                            &[path.to_path_buf()],
                            &app_ui,
                            &mymod_stuff,
                            &mode,
                            "",
                            &packedfiles_open_in_packedfile_view,
                            close_global_search_action,
                            &table_state_data,
                        ) { show_dialog(app_ui.window, false, error); }
                    }
                }
            ));

            // Add the slot to the list.
            open_from_slots.push(slot_open_mod);

            // Connect the action to the slot.
            unsafe { open_mod_action.as_ref().unwrap().signals().triggered().connect(open_from_slots.last().unwrap()); }  
        }
    }
    
    // Only if the submenu has items, we enable it.
    unsafe { submenu_open_from_content.as_mut().unwrap().menu_action().as_mut().unwrap().set_visible(!submenu_open_from_content.as_mut().unwrap().actions().is_empty()); }
    unsafe { submenu_open_from_data.as_mut().unwrap().menu_action().as_mut().unwrap().set_visible(!submenu_open_from_data.as_mut().unwrap().actions().is_empty()); }

    // Return the slots.
    open_from_slots
}

/// This function is the one that takes care of the creation of different PackedFiles.
pub fn create_packed_files(
    sender_qt: &Sender<Commands>,
    sender_qt_data: &Sender<Data>,
    receiver_qt: &Rc<RefCell<Receiver<Data>>>,
    table_state_data: &Rc<RefCell<BTreeMap<Vec<String>, TableStateData>>>,
    app_ui: &AppUI,
    packed_file_type: &PackedFileType,
) {
    // Create the "New PackedFile" dialog and wait for his data (or a cancelation). If we receive None, we do nothing. If we receive Some,
    // we still have to check if it has been any error during the creation of the PackedFile (for example, no definition for DB Tables).
    if let Some(packed_file_type) = create_new_packed_file_dialog(&app_ui, &sender_qt, &sender_qt_data, &receiver_qt, packed_file_type) {
        match packed_file_type {
            Ok(packed_file_type) => {

                // If we reach this place, we got all alright. Now act depending on the type of PackedFile we want to create.
                match packed_file_type.clone() {
                    PackedFileType::Loc(ref mut name) | PackedFileType::Text(ref mut name) | PackedFileType::DB(ref mut name, _, _) => {

                        // If the name is_empty, stop.
                        if name.is_empty() { return show_dialog(app_ui.window, false, ErrorKind::EmptyInput) }

                        // Fix their name termination if needed.
                        if let PackedFileType::Loc(_) = packed_file_type {
                            if !name.ends_with(".loc") { name.push_str(".loc"); }
                        }
                        if let PackedFileType::Text(_) = packed_file_type {
                            if !name.ends_with(".lua") &&
                                !name.ends_with(".xml") &&
                                !name.ends_with(".xml.shader") &&
                                !name.ends_with(".xml.material") &&
                                !name.ends_with(".variantmeshdefinition") &&
                                !name.ends_with(".environment") &&
                                !name.ends_with(".lighting") &&
                                !name.ends_with(".wsmodel") &&
                                !name.ends_with(".csv") &&
                                !name.ends_with(".tsv") &&
                                !name.ends_with(".inl") &&
                                !name.ends_with(".battle_speech_camera") &&
                                !name.ends_with(".bob") &&
                                !name.ends_with(".cindyscene") &&
                                !name.ends_with(".cindyscenemanager") &&
                                !name.ends_with(".txt") {
                                name.push_str(".txt");
                            }
                        }

                        // Get the currently selected paths (or the complete path, in case of DB Tables),
                        // and only continue if there is only one and it's not empty.
                        let selected_paths = get_path_from_main_treeview_selection(&app_ui);
                        let complete_path = if let PackedFileType::DB(_, table,_) = &packed_file_type {
                            vec!["db".to_owned(), table.to_owned(), name.to_owned()]
                        } 
                        else {

                            // We want to be able to write relative paths with this so, if a `/` is detected, split the name.
                            if selected_paths.len() == 1 { 
                                let mut complete_path = selected_paths[0].to_vec();
                                complete_path.append(&mut (name.split("/").map(|x| x.to_owned()).filter(|x| !x.is_empty()).collect::<Vec<String>>()));
                                complete_path
                            }
                            else { vec![] }
                        };

                        // If and only if, after all these checks, we got a path to save the PackedFile, we continue.
                        if !complete_path.is_empty() {

                            // Check if the PackedFile already exists, and report it if so.
                            sender_qt.send(Commands::PackedFileExists).unwrap();
                            sender_qt_data.send(Data::VecString(complete_path.to_vec())).unwrap();
                            let exists = if let Data::Bool(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };
                            if exists { return show_dialog(app_ui.window, false, ErrorKind::FileAlreadyInPackFile)}

                            // Add it to the PackFile.
                            sender_qt.send(Commands::CreatePackedFile).unwrap();
                            sender_qt_data.send(Data::VecStringPackedFileType((complete_path.to_vec(), packed_file_type.clone()))).unwrap();

                            // Get the response, just in case it failed.
                            match check_message_validity_recv2(&receiver_qt) {
                                Data::Success => {
                                    
                                    // Add the new Folder to the TreeView.
                                    update_treeview(
                                        &sender_qt,
                                        &sender_qt_data,
                                        &receiver_qt,
                                        &app_ui,
                                        app_ui.folder_tree_view,
                                        Some(app_ui.folder_tree_filter),
                                        app_ui.folder_tree_model,
                                        TreeViewOperation::Add(vec![TreePathType::File(complete_path.to_vec()); 1]),
                                    );

                                    // If, for some reason, there is a TableState data for this file, remove it.
                                    if table_state_data.borrow().get(&complete_path).is_some() {
                                        table_state_data.borrow_mut().remove(&complete_path);
                                    }

                                    // Set it to not remove his color.
                                    let data = TableStateData::new_empty();
                                    table_state_data.borrow_mut().insert(complete_path, data);
                                }

                                Data::Error(error) => show_dialog(app_ui.window, false, error),

                                // In ANY other situation, it's a message problem.
                                _ => panic!(THREADS_MESSAGE_ERROR),
                            }
                        }
                    }
                }
            }

            // If we got an error while trying to prepare the dialog, report it.
            Err(error) => show_dialog(app_ui.window, false, error),
        }
    }
}

/// This function enables or disables the actions from the `MenuBar` needed when we open a PackFile.
/// NOTE: To disable the "Special Stuff" actions, we use `enable` => false.
pub fn enable_packfile_actions(
    app_ui: &AppUI,
    mymod_stuff: &Rc<RefCell<MyModStuff>>,
    enable: bool
) {

    // If the game is Arena, no matter what we're doing, these ones ALWAYS have to be disabled.
    if &**GAME_SELECTED.lock().unwrap() == "arena" {

        // Disable the actions that allow to create and save PackFiles.
        unsafe { app_ui.new_packfile.as_mut().unwrap().set_enabled(false); }
        unsafe { app_ui.save_packfile.as_mut().unwrap().set_enabled(false); }
        unsafe { app_ui.save_packfile_as.as_mut().unwrap().set_enabled(false); }

        // This one too, though we had to deal with it specially later on.
        unsafe { mymod_stuff.borrow().new_mymod.as_mut().unwrap().set_enabled(false); }
    }

    // Otherwise...
    else {

        // Enable or disable the actions from "PackFile" Submenu.
        unsafe { app_ui.new_packfile.as_mut().unwrap().set_enabled(true); }
        unsafe { app_ui.save_packfile.as_mut().unwrap().set_enabled(enable); }
        unsafe { app_ui.save_packfile_as.as_mut().unwrap().set_enabled(enable); }

        // If there is a "MyMod" path set in the settings...
        if let Some(ref path) = SETTINGS.lock().unwrap().paths["mymods_base_path"] {

            // And it's a valid directory, enable the "New MyMod" button.
            if path.is_dir() { unsafe { mymod_stuff.borrow().new_mymod.as_mut().unwrap().set_enabled(true); }}

            // Otherwise, disable it.
            else { unsafe { mymod_stuff.borrow().new_mymod.as_mut().unwrap().set_enabled(false); }}
        }

        // Otherwise, disable it.
        else { unsafe { mymod_stuff.borrow().new_mymod.as_mut().unwrap().set_enabled(false); }}
    }

    // These actions are common, no matter what game we have.    
    unsafe { app_ui.change_packfile_type_group.as_mut().unwrap().set_enabled(enable); }
    unsafe { app_ui.change_packfile_type_index_includes_timestamp.as_mut().unwrap().set_enabled(enable); }

    // If we are enabling...
    if enable {

        // Check the Game Selected and enable the actions corresponding to out game.
        match &**GAME_SELECTED.lock().unwrap() {
            "three_kingdoms" => {
                unsafe { app_ui.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(true); }
                unsafe { app_ui.three_k_optimize_packfile.as_mut().unwrap().set_enabled(true); }
                unsafe { app_ui.three_k_generate_pak_file.as_mut().unwrap().set_enabled(true); }
            },
            "warhammer_2" => {
                unsafe { app_ui.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(true); }
                unsafe { app_ui.wh2_patch_siege_ai.as_mut().unwrap().set_enabled(true); }
                unsafe { app_ui.wh2_optimize_packfile.as_mut().unwrap().set_enabled(true); }
                unsafe { app_ui.wh2_generate_pak_file.as_mut().unwrap().set_enabled(true); }
            },
            "warhammer" => {
                unsafe { app_ui.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(false); }
                unsafe { app_ui.wh_patch_siege_ai.as_mut().unwrap().set_enabled(true); }
                unsafe { app_ui.wh_optimize_packfile.as_mut().unwrap().set_enabled(true); }
                unsafe { app_ui.wh_generate_pak_file.as_mut().unwrap().set_enabled(true); }
            },
            "thrones_of_britannia" => {
                unsafe { app_ui.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(false); }
                unsafe { app_ui.tob_optimize_packfile.as_mut().unwrap().set_enabled(true); }
                unsafe { app_ui.tob_generate_pak_file.as_mut().unwrap().set_enabled(true); }
            },
            "attila" => {
                unsafe { app_ui.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(false); }
                unsafe { app_ui.att_optimize_packfile.as_mut().unwrap().set_enabled(true); }
                unsafe { app_ui.att_generate_pak_file.as_mut().unwrap().set_enabled(true); }
            },
            "rome_2" => {
                unsafe { app_ui.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(false); }
                unsafe { app_ui.rom2_optimize_packfile.as_mut().unwrap().set_enabled(true); }
                unsafe { app_ui.rom2_generate_pak_file.as_mut().unwrap().set_enabled(true); }
            },
            "shogun_2" => {
                unsafe { app_ui.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(false); }
                unsafe { app_ui.sho2_optimize_packfile.as_mut().unwrap().set_enabled(true); }
                unsafe { app_ui.sho2_generate_pak_file.as_mut().unwrap().set_enabled(true); }
            },
            "napoleon" => {
                unsafe { app_ui.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(false); }
                unsafe { app_ui.nap_optimize_packfile.as_mut().unwrap().set_enabled(true); }
                unsafe { app_ui.nap_generate_pak_file.as_mut().unwrap().set_enabled(true); }
            },
            "empire" => {
                unsafe { app_ui.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(false); }
                unsafe { app_ui.emp_optimize_packfile.as_mut().unwrap().set_enabled(true); }
                unsafe { app_ui.emp_generate_pak_file.as_mut().unwrap().set_enabled(true); }
            },
            _ => {},
        }
    }

    // If we are disabling...
    else {

        // Universal Actions.
        unsafe { app_ui.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(false); }

        // Disable Three Kingdoms actions...
        unsafe { app_ui.three_k_optimize_packfile.as_mut().unwrap().set_enabled(false); }
        unsafe { app_ui.three_k_generate_pak_file.as_mut().unwrap().set_enabled(false); }

        // Disable Warhammer 2 actions...
        unsafe { app_ui.wh2_patch_siege_ai.as_mut().unwrap().set_enabled(false); }
        unsafe { app_ui.wh2_optimize_packfile.as_mut().unwrap().set_enabled(false); }
        unsafe { app_ui.wh2_generate_pak_file.as_mut().unwrap().set_enabled(false); }

        // Disable Warhammer actions...
        unsafe { app_ui.wh_patch_siege_ai.as_mut().unwrap().set_enabled(false); }
        unsafe { app_ui.wh_optimize_packfile.as_mut().unwrap().set_enabled(false); }
        unsafe { app_ui.wh_generate_pak_file.as_mut().unwrap().set_enabled(false); }

        // Disable Thrones of Britannia actions...
        unsafe { app_ui.tob_optimize_packfile.as_mut().unwrap().set_enabled(false); }
        unsafe { app_ui.tob_generate_pak_file.as_mut().unwrap().set_enabled(false); }

        // Disable Attila actions...
        unsafe { app_ui.att_optimize_packfile.as_mut().unwrap().set_enabled(false); }
        unsafe { app_ui.att_generate_pak_file.as_mut().unwrap().set_enabled(false); }

        // Disable Rome 2 actions...
        unsafe { app_ui.rom2_optimize_packfile.as_mut().unwrap().set_enabled(false); }
        unsafe { app_ui.rom2_generate_pak_file.as_mut().unwrap().set_enabled(false); }

        // Disable Shogun 2 actions...
        unsafe { app_ui.sho2_optimize_packfile.as_mut().unwrap().set_enabled(false); }
        unsafe { app_ui.sho2_generate_pak_file.as_mut().unwrap().set_enabled(false); }

        // Disable Napoleon actions...
        unsafe { app_ui.nap_optimize_packfile.as_mut().unwrap().set_enabled(false); }
        unsafe { app_ui.nap_generate_pak_file.as_mut().unwrap().set_enabled(false); }
        
        // Disable Empire actions...
        unsafe { app_ui.emp_optimize_packfile.as_mut().unwrap().set_enabled(false); }
        unsafe { app_ui.emp_generate_pak_file.as_mut().unwrap().set_enabled(false); }
    }

    // The assembly kit thing should only be available for Rome 2 and later games.
    match &**GAME_SELECTED.lock().unwrap() {
        "three_kingdoms" |
        "warhammer_2" |
        "warhammer" |
        "thrones_of_britannia" |
        "attila" |
        "rome_2" => unsafe { app_ui.open_game_assembly_kit_folder.as_mut().unwrap().set_enabled(true); }
        _ => unsafe { app_ui.open_game_assembly_kit_folder.as_mut().unwrap().set_enabled(false); }
    }
}

/// This function is used to set the current "Operational Mode". It not only sets the "Operational Mode",
/// but it also takes care of disabling or enabling all the signals related with the "MyMod" Mode.
/// If `my_mod_path` is None, we want to set the `Normal` mode. Otherwise set the `MyMod` mode.
pub fn set_my_mod_mode(
    mymod_stuff: &Rc<RefCell<MyModStuff>>,
    mode: &Rc<RefCell<Mode>>,
    my_mod_path: Option<PathBuf>,
) {
    // Check if we provided a "my_mod_path".
    match my_mod_path {

        // If we have a `my_mod_path`...
        Some(my_mod_path) => {

            // Get the `folder_name` and the `mod_name` of our "MyMod".
            let mut path = my_mod_path.clone();
            let mod_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
            path.pop();
            let game_folder_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();

            // Set the current mode to `MyMod`.
            *mode.borrow_mut() = Mode::MyMod {
                game_folder_name,
                mod_name,
            };

            // Enable all the "MyMod" related actions.
            unsafe { mymod_stuff.borrow_mut().delete_selected_mymod.as_mut().unwrap().set_enabled(true); }
            unsafe { mymod_stuff.borrow_mut().install_mymod.as_mut().unwrap().set_enabled(true); }
            unsafe { mymod_stuff.borrow_mut().uninstall_mymod.as_mut().unwrap().set_enabled(true); }
        }

        // If `None` has been provided...
        None => {

            // Set the current mode to `Normal`.
            *mode.borrow_mut() = Mode::Normal;

            // Disable all "MyMod" related actions, except "New MyMod".
            unsafe { mymod_stuff.borrow_mut().delete_selected_mymod.as_mut().unwrap().set_enabled(false); }
            unsafe { mymod_stuff.borrow_mut().install_mymod.as_mut().unwrap().set_enabled(false); }
            unsafe { mymod_stuff.borrow_mut().uninstall_mymod.as_mut().unwrap().set_enabled(false); }
        }
    }
}

/// Function to filter the results of a global search, in any of the result tables.
/// If a value is not provided by a slot, we get it from the widget itself.
pub fn filter_matches_result(
    pattern: Option<QString>,
    column: Option<i32>,
    case_sensitive: Option<bool>,
    filter_model: *mut SortFilterProxyModel,
    filter_line_edit: *mut LineEdit,
    column_selector: *mut ComboBox,
    case_sensitive_button: *mut PushButton,
) {

    // Set the pattern to search.
    let mut pattern = if let Some(pattern) = pattern { RegExp::new(&pattern) }
    else { 
        let pattern;
        unsafe { pattern = RegExp::new(&filter_line_edit.as_mut().unwrap().text()) }
        pattern
    };

    // Set the column selected.
    if let Some(column) = column { unsafe { filter_model.as_mut().unwrap().set_filter_key_column(column); }}
    else { unsafe { filter_model.as_mut().unwrap().set_filter_key_column(column_selector.as_mut().unwrap().current_index()); }}

    // Check if the filter should be "Case Sensitive".
    if let Some(case_sensitive) = case_sensitive { 
        if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::Sensitive); }
        else { pattern.set_case_sensitivity(CaseSensitivity::Insensitive); }
    }

    else {
        unsafe { 
            let case_sensitive = case_sensitive_button.as_mut().unwrap().is_checked();
            if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::Sensitive); }
            else { pattern.set_case_sensitivity(CaseSensitivity::Insensitive); }
        }
    }

    // Filter whatever it's in that column by the text we got.
    unsafe { filter_model.as_mut().unwrap().set_filter_reg_exp(&pattern); }
}

/// Function to filter the file list. If a value is not provided by a slot, we get it from the widget itself.
pub fn filter_files(app_ui: &AppUI) {

    // Set the pattern to search.
    let mut pattern = unsafe { RegExp::new(&app_ui.folder_tree_filter_line_edit.as_mut().unwrap().text()) };

    // Check if the filter should be "Case Sensitive" and if it should "Filter By Folders".
    let filter_by_folder = unsafe { app_ui.folder_tree_filter_filter_by_folder_button.as_mut().unwrap().is_checked() };
    let case_sensitive = unsafe { app_ui.folder_tree_filter_case_sensitive_button.as_mut().unwrap().is_checked() };
    if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::Sensitive); }
    else { pattern.set_case_sensitivity(CaseSensitivity::Insensitive); }

    // Filter whatever it's in that column by the text we got.
    unsafe { trigger_treeview_filter(app_ui.folder_tree_filter, &mut pattern, filter_by_folder); }

    // Expand all the matches, if the option for it is enabled.
    if unsafe { app_ui.folder_tree_filter_autoexpand_matches_button.as_ref().unwrap().is_checked() } {
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().expand_all(); }
    }
}
