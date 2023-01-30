#![feature(proc_macro_hygiene)]

use skyline::{hook, install_hook};
use smash::phx::*;
use smash::hash40;
use smash::app::{self, lua_bind::*};
use smash::lib::lua_const::*;

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
        return;
    }

    if ControlModule::button_on(CONTROL_PAD_BUTTON_STOCK_SHARE) {
        if fighter.is_button_on(CONTROL_PAD_BUTTON_ATTACK_RAW) && !fighter.is_button_on(!(CONTROL_PAD_BUTTON_ATTACK_RAW | CONTROL_PAD_BUTTON_STOCK_SHARE)) {
            app::FighterUtil::flash_eye_info(fighter.module_accessor);
            EffectModule::req_follow(fighter.module_accessor, Hash40::new("sys_assist_out"), Hash40::new("top"), &Vector3f{x: 0, y: 0, z: 0}, &Vector3f{x: 0, y: 0, z: 0}, 1.0, true, 0, 0, 0, 0, 0, false, false);
            utils::util::trigger_match_reset();
        }
    }

    flag
}

#[skyline::main(name = "salty_runback")]
pub fn main() {
    println!("[Salty Runback] Initializing...");
    install_hook!(handle_get_command_flag_cat);
    println!("[Salty Runback] Installed!");
}
