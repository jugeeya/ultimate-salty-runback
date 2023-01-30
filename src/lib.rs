#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate modular_bitfield;

use skyline::install_hooks;
use smash::phx::*;
use smash::app::{self, lua_bind::*};
use smash::lib::lua_const::*;
use input::{ControllerMapping, MappedInputs, SomeControllerStruct, Buttons}

/// Taken from HDR: https://github.com/HDR-Development/HewDraw-Remix/blob/76140b549c829ceaf8d6b7fa5ce4b42bd99bf57d/dynamic/src/util.rs#L230-L242
pub const fn p_p_game_state() -> usize {
    0x52c1760
}

pub fn offset_to_addr<T>(offset: usize) -> *const T {
    unsafe {
        (skyline::hooks::getRegionAddress(skyline::hooks::Region::Text) as *const u8).add(offset) as _
    }
}

/// Only pulls the game state to perform actions on
pub fn get_game_state() -> *const u64 {
    unsafe {
        let p_p_p_game_state = *offset_to_addr::<*const *const *const u64>(p_p_game_state());
        if p_p_p_game_state.is_null() {
            return std::ptr::null();
        }
        let p_p_game_state = *p_p_p_game_state;
        if p_p_game_state.is_null() {
            return std::ptr::null();
        }
        let p_game_state = *p_p_game_state;
        if p_game_state.is_null() {
            return std::ptr::null();
        }
        p_game_state
    }
}

/// Triggers a match reset by loading into the same state that classic mode uses when you retry a game
/// Note: Calling this function outside of a match shouldn't crash but it has undefined behavior. If you do that, don't
pub fn trigger_match_reset() {
    unsafe {
        let p_game_state = get_game_state();
        if p_game_state.is_null() {
            return;
        }
        // Finally call the vtable function on the game state
        let vtable_func: extern "C" fn(*const u64) = std::mem::transmute(*(*p_game_state as *const u64).add(0x5));
        vtable_func(p_game_state);
    }
}

#[skyline::hook(replace = ControlModule::get_command_flag_cat)]
pub unsafe fn handle_get_command_flag_cat(
    module_accessor: &mut app::BattleObjectModuleAccessor,
    category: i32,
) -> i32 {
    let mut flag = original!()(module_accessor, category);

    if category != FIGHTER_PAD_COMMAND_CATEGORY1 {
        return flag;
    }

    if is_button_on(module_accessor, Buttons::StockShare) {
        if is_button_on(module_accessor, Buttons::AttackRaw) && !is_button_on(module_accessor, !(Buttons::AttackRaw | Buttons::StockShare)) {
            app::FighterUtil::flash_eye_info(module_accessor);
            EffectModule::req_follow(module_accessor, Hash40::new("sys_assist_out"), Hash40::new("top"), &Vector3f{x: 0.0, y: 0.0, z: 0.0}, &Vector3f{x: 0.0, y: 0.0, z: 0.0}, 1.0, true, 0, 0, 0, 0, 0, false, false);
            trigger_match_reset();
        } 
    }

    flag
}


#[skyline::hook(offset = 0x17504a0)]
unsafe fn map_controls_hook(
    mappings: *mut ControllerMapping,
    player_idx: i32,
    out: *mut MappedInputs,
    controller_struct: &SomeControllerStruct,
    arg: bool
) {
    let controller = controller_struct.controller;
    let ret = original!()(mappings, player_idx, out, controller_struct, arg);

    // Check if the button combos are being pressed and then force Stock Share + AttackRaw/SpecialRaw depending on input

    if controller.current_buttons.l()
    && controller.current_buttons.r()
    && controller.current_buttons.a()
    && (controller.current_buttons.minus() || controller.current_buttons.plus())
    {
        if controller.current_buttons.x() {
            (*out).buttons = Buttons::StockShare | Buttons::AttackRaw;
        } else if controller.current_buttons.y() {
            (*out).buttons = Buttons::StockShare | Buttons::SpecialRaw;
        }
    }
}

#[skyline::main(name = "salty_runback")]
pub fn main() {
    println!("[Salty Runback] Initializing...");
    install_hooks!(
        handle_get_command_flag_cat,
        map_controls_hook
    );
    println!("[Salty Runback] Installed!");
}
