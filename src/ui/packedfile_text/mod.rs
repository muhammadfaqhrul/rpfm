//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// In this file are all the helper functions used by the UI when editing Text PackedFiles.

use qt_widgets::action::Action;
use qt_widgets::dialog::Dialog;
use qt_widgets::dialog_button_box::{DialogButtonBox, StandardButton};
use qt_widgets::plain_text_edit::PlainTextEdit;
use qt_widgets::widget::Widget;

use qt_core::connection::Signal;

use std::cell::RefCell;
use std::rc::Rc;

use crate::SUPPORTED_GAMES;
use crate::GAME_SELECTED;
use crate::AppUI;
use crate::Commands;
use crate::Data;
use crate::common::communications::*;
use crate::ui::*;
use crate::error::Result;

pub mod packedfile_text;
pub mod packfile_notes;

//----------------------------------------------------------------//
// Generic Enums and Structs for Text PackedFiles.
//----------------------------------------------------------------//

/// Enum `TableType`: used to distinguis between DB and Loc.
#[derive(Clone)]
pub enum TextType {
    PackedFile(String),
    Notes(String),
}

/// Struct `PackedFileTextView`: contains all the stuff we need to give to the program to show a
/// `PlainTextEdit` with the data of a plain text PackedFile, allowing us to manipulate it.
pub struct PackedFileTextView {
    pub save_changes: SlotNoArgs<'static>,
    pub check_syntax: SlotNoArgs<'static>,
    pub close_note: SlotNoArgs<'static>,
    pub close_note_action: *mut Action,
}

//----------------------------------------------------------------//
// Implementation of `PackedFileTextView`.
//----------------------------------------------------------------//

/// Implementation of `PackedFileTextView`.
impl PackedFileTextView {

    /// This function creates a new TreeView with the PackedFile's View as father and returns a
    /// `PackedFileTextView` with all his data.
    pub fn create_text_view(
        sender_qt: &Sender<Commands>,
        sender_qt_data: &Sender<Data>,
        receiver_qt: &Rc<RefCell<Receiver<Data>>>,
        app_ui: &AppUI,
        layout: *mut GridLayout,
        packed_file_path: &Rc<RefCell<Vec<String>>>,
        packedfiles_open_in_packedfile_view: &Rc<RefCell<BTreeMap<i32, Rc<RefCell<Vec<String>>>>>>,
        text_type: &Rc<RefCell<TextType>>,
    ) -> Result<Self> {

        let text = match *text_type.borrow() {
            TextType::PackedFile(ref text) => text.to_owned(),
            TextType::Notes(ref text) => text.to_owned(),
        };

        // Create the PlainTextEdit and the checking button.
        let plain_text_edit = PlainTextEdit::new(&QString::from_std_str(&text)).into_raw();
        let check_syntax_button = PushButton::new(&QString::from_std_str("Check Syntax")).into_raw();
        let close_button = PushButton::new(&QString::from_std_str("Close Note")).into_raw();

        // Add it to the view.
        unsafe { layout.as_mut().unwrap().add_widget((plain_text_edit as *mut Widget, 0, 0, 1, 1)); }
        if let TextType::PackedFile(_) = *text_type.borrow() {
            if packed_file_path.borrow().last().unwrap().ends_with(".lua") && SUPPORTED_GAMES.get(&**GAME_SELECTED.lock().unwrap()).unwrap().ca_types_file.is_some() {
                unsafe { layout.as_mut().unwrap().add_widget((check_syntax_button as *mut Widget, 1, 0, 1, 1)); }
            }
        }

        // Create the stuff needed for this to work.
        let stuff = Self {
            save_changes: SlotNoArgs::new(clone!(
                packed_file_path,
                app_ui,
                text_type,
                receiver_qt,
                sender_qt,
                sender_qt_data => move || {

                    // Get the text from the PlainTextEdit and save it, depending on his type.
                    let text = unsafe { plain_text_edit.as_mut().unwrap().to_plain_text().to_std_string() };
                    match *text_type.borrow() {
                        TextType::PackedFile(_) => {
                            sender_qt.send(Commands::EncodePackedFileText).unwrap();
                            sender_qt_data.send(Data::StringVecString((text, packed_file_path.borrow().to_vec()))).unwrap();

                            update_treeview(
                                &sender_qt,
                                &sender_qt_data,
                                &receiver_qt,
                                &app_ui,
                                app_ui.folder_tree_view,
                                Some(app_ui.folder_tree_filter),
                                app_ui.folder_tree_model,
                                TreeViewOperation::Modify(vec![TreePathType::File(packed_file_path.borrow().to_vec())]),
                            );
                        },
                        TextType::Notes(_) => {
                            sender_qt.send(Commands::SetNotes).unwrap();
                            sender_qt_data.send(Data::String(text)).unwrap();

                            update_treeview(
                                &sender_qt,
                                &sender_qt_data,
                                &receiver_qt,
                                &app_ui,
                                app_ui.folder_tree_view,
                                Some(app_ui.folder_tree_filter),
                                app_ui.folder_tree_model,
                                TreeViewOperation::Modify(vec![TreePathType::PackFile]),
                            );

                            // This has to mark the PackFile as impossible to undo.
                            update_treeview(
                                &sender_qt,
                                &sender_qt_data,
                                &receiver_qt,
                                &app_ui,
                                app_ui.folder_tree_view,
                                Some(app_ui.folder_tree_filter),
                                app_ui.folder_tree_model,
                                TreeViewOperation::MarkAlwaysModified(vec![TreePathType::PackFile]),
                            );
                        }
                    }
                }
            )),

            check_syntax: SlotNoArgs::new(clone!(
                app_ui,
                sender_qt,
                receiver_qt => move || {

                    // Tell the background thread to check the PackedFile, and return the result.
                    sender_qt.send(Commands::CheckScriptWithKailua).unwrap();
                    let result = match check_message_validity_recv2(&receiver_qt) { 
                        Data::VecString(data) => data,
                        Data::Error(error) => return show_dialog(app_ui.window, false, error),
                        _ => panic!(THREADS_MESSAGE_ERROR), 
                    };

                    let mut clean_result = String::new();
                    result.iter().for_each(|x| clean_result.push_str(&format!("{}\n", x)));

                    // Create the dialog.
                    let dialog = unsafe { Dialog::new_unsafe(app_ui.window as *mut Widget).into_raw() };

                    // Create the Grid.
                    let grid = create_grid_layout_unsafe(dialog as *mut Widget);

                    // Configure the dialog.
                    unsafe { dialog.as_mut().unwrap().set_window_title(&QString::from_std_str("Script Checked!")); }
                    unsafe { dialog.as_mut().unwrap().set_modal(false); }
                    unsafe { dialog.as_mut().unwrap().resize((950, 500)); }

                    // Create the Text View and the ButtonBox.
                    let mut error_report = PlainTextEdit::new(&QString::from_std_str(clean_result));
                    let mut button_box = DialogButtonBox::new(());
                    error_report.set_read_only(true);
                    let close_button = button_box.add_button(StandardButton::Close);
                    unsafe { close_button.as_mut().unwrap().signals().released().connect(&dialog.as_mut().unwrap().slots().close()); }
                    unsafe { grid.as_mut().unwrap().add_widget((error_report.into_raw() as *mut Widget, 0, 0, 1, 1)); }
                    unsafe { grid.as_mut().unwrap().add_widget((button_box.into_raw() as *mut Widget, 1, 0, 1, 1)); }

                    // Show the Dialog, so it doesn't block the program.
                    unsafe { dialog.as_mut().unwrap().show(); }
                }
            )),
            close_note: SlotNoArgs::new(clone!(
                packedfiles_open_in_packedfile_view,
                app_ui => move || {
                    purge_that_one_specifically(&app_ui, 1, &packedfiles_open_in_packedfile_view); 
                    let widgets = unsafe { app_ui.packed_file_splitter.as_mut().unwrap().count() };
                    let visible_widgets = (0..widgets).filter(|x| unsafe {app_ui.packed_file_splitter.as_mut().unwrap().widget(*x).as_mut().unwrap().is_visible() } ).count();
                    if visible_widgets == 0 { display_help_tips(&app_ui); }
                }
            )),
            close_note_action: Action::new(&QString::from_std_str("&Close")).into_raw(),
        };

        // Actions to trigger the slots.
        unsafe { plain_text_edit.as_ref().unwrap().signals().text_changed().connect(&stuff.save_changes); }
        unsafe { check_syntax_button.as_ref().unwrap().signals().released().connect(&stuff.check_syntax); }

        // If it's a note, add the close button to the view.
        if let TextType::Notes(_) = *text_type.borrow() {
            unsafe { layout.as_mut().unwrap().add_widget((close_button as *mut Widget, 1, 0, 1, 1)); }

            // Connect the close signal to the button. Also, we want to trigger it with the same "open_notes" shortcut from the text view.
            unsafe { close_button.as_ref().unwrap().signals().released().connect(&stuff.close_note); }
            unsafe { stuff.close_note_action.as_ref().unwrap().signals().triggered().connect(&stuff.close_note); }
            unsafe { stuff.close_note_action.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["open_notes"]))); }
            unsafe { stuff.close_note_action.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
            unsafe { plain_text_edit.as_mut().unwrap().add_action(stuff.close_note_action); }
        }

        // Return the slots.
        Ok(stuff)
    }
}
