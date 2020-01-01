//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code related to the `MyModUISlots`.
!*/

use qt_core::slots::SlotNoArgs;

use crate::mymod_ui::MyModUI;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the slots we need to respond to signals of the New MyMod Dialog.
pub struct MyModUISlots {
    pub mymod_name_change: SlotNoArgs<'static>,
    pub mymod_game_change: SlotNoArgs<'static>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `MyModUISlots`.
impl MyModUISlots {

    /// This function creates an entire `MyModUISlots` struct.
    pub fn new(mymod_ui: MyModUI) -> Self {

        // What happens when we change the name of the MyMod.
        let mymod_name_change = SlotNoArgs::new(move || {
            mymod_ui.check_my_mod_validity();
        });

        // What happens when we change the game the Mymod is for.
        let mymod_game_change = SlotNoArgs::new(move || {
            mymod_ui.check_my_mod_validity();
        });

        // And here... we return all the slots.
        Self {
            mymod_name_change,
            mymod_game_change,
        }
    }
}
