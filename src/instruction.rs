use crate::choices::{Choices, MoveCategory};
use crate::engine::abilities::Abilities;
use crate::engine::items::Items;
use crate::engine::state::{PokemonVolatileStatus, Terrain, Weather};
use crate::state::{
    LastUsedMove, PokemonBoostableStat, PokemonIndex, PokemonMoveIndex, PokemonSideCondition,
    PokemonStatus, PokemonType, SideReference,
};
use std::fmt;
use std::fmt::Formatter;

#[derive(PartialEq, Clone)]
pub struct StateInstructions {
    pub percentage: f32,
    pub instruction_list: Vec<Instruction>,
}

impl Default for StateInstructions {
    fn default() -> StateInstructions {
        StateInstructions {
            percentage: 100.0,
            instruction_list: Vec::with_capacity(4),
        }
    }
}

impl fmt::Debug for StateInstructions {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut final_string = format!("\n\tPercentage: {}\n\tInstructions:", self.percentage);
        for i in self.instruction_list.iter() {
            final_string.push_str(format!("\n\t\t{:?}", i).as_str());
        }
        write!(f, "{}\n", final_string)
    }
}

impl StateInstructions {
    pub fn update_percentage(&mut self, modifier: f32) {
        self.percentage *= modifier;
    }
}

// https://stackoverflow.com/questions/50686411/whats-the-usual-way-to-create-a-vector-of-different-structs
#[derive(Clone)]
pub enum Instruction {
    Switch(SwitchInstruction),
    ApplyVolatileStatus(ApplyVolatileStatusInstruction),
    RemoveVolatileStatus(RemoveVolatileStatusInstruction),
    ChangeStatus(ChangeStatusInstruction),
    Heal(HealInstruction),
    Damage(DamageInstruction),
    DamageWithFaintContext(DamageWithFaintContextInstruction),
    Boost(BoostInstruction),
    ChangeSideCondition(ChangeSideConditionInstruction),
    ChangeVolatileStatusDuration(ChangeVolatileStatusDurationInstruction),
    ChangeWeather(ChangeWeather),
    DecrementWeatherTurnsRemaining,
    ChangeTerrain(ChangeTerrain),
    DecrementTerrainTurnsRemaining,
    ChangeType(ChangeType),
    ChangeAbility(ChangeAbilityInstruction),
    ChangeBaseAbility(ChangeAbilityInstruction),
    ChangeItem(ChangeItemInstruction),
    ChangeAttack(ChangeStatInstruction),
    ChangeDefense(ChangeStatInstruction),
    ChangeSpecialAttack(ChangeStatInstruction),
    ChangeSpecialDefense(ChangeStatInstruction),
    ChangeSpeed(ChangeStatInstruction),
    DisableMove(DisableMoveInstruction),
    EnableMove(EnableMoveInstruction),
    ChangeWish(ChangeWishInstruction),
    DecrementWish(DecrementWishInstruction),
    SetFutureSight(SetFutureSightInstruction),
    DecrementFutureSight(DecrementFutureSightInstruction),
    DamageSubstitute(DamageInstruction),
    DecrementRestTurns(DecrementRestTurnsInstruction),
    SetRestTurns(SetSleepTurnsInstruction),
    SetSleepTurns(SetSleepTurnsInstruction),
    SetFreezeTurns(SetSleepTurnsInstruction),
    ChangeSubstituteHealth(ChangeSubsituteHealthInstruction),
    FormeChange(FormeChangeInstruction),
    SetSideOneMoveSecondSwitchOutMove(SetSecondMoveSwitchOutMoveInstruction),
    SetSideTwoMoveSecondSwitchOutMove(SetSecondMoveSwitchOutMoveInstruction),
    ToggleBatonPassing(ToggleBatonPassingInstruction),
    ToggleShedTailing(ToggleShedTailingInstruction),
    SetLastUsedMove(SetLastUsedMoveInstruction),
    ChangeDamageDealtDamage(ChangeDamageDealtDamageInstruction),
    ChangeDamageDealtMoveCatagory(ChangeDamageDealtMoveCategoryInstruction),
    ToggleDamageDealtHitSubstitute(ToggleDamageDealtHitSubstituteInstruction),
    DecrementPP(DecrementPPInstruction),
    ToggleTrickRoom(ToggleTrickRoomInstruction),
    DecrementTrickRoomTurnsRemaining,
    ToggleSideOneForceSwitch,
    ToggleSideTwoForceSwitch,
    ToggleTerastallized(ToggleTerastallizedInstruction),
    ToggleMegaEvolved(ToggleMegaEvolvedInstruction),
    ToggleSwordBoostUsed(ToggleAbilityOnStartFlagInstruction),
    ToggleShieldBoostUsed(ToggleAbilityOnStartFlagInstruction),
    TeamPreview(TeamPreviewInstruction),
}

impl PartialEq for Instruction {
    fn eq(&self, other: &Self) -> bool {
        use Instruction::*;
        match (self, other) {
            (Switch(a), Switch(b)) => a == b,
            (ApplyVolatileStatus(a), ApplyVolatileStatus(b)) => a == b,
            (RemoveVolatileStatus(a), RemoveVolatileStatus(b)) => a == b,
            (ChangeStatus(a), ChangeStatus(b)) => a == b,
            (Heal(a), Heal(b)) => a == b,
            (Damage(a), Damage(b)) => a == b,
            (DamageWithFaintContext(a), DamageWithFaintContext(b)) => a == b,
            (Damage(a), DamageWithFaintContext(b)) | (DamageWithFaintContext(b), Damage(a)) => {
                a.side_ref == b.side_ref && a.damage_amount == b.damage_amount
            }
            (Boost(a), Boost(b)) => a == b,
            (ChangeSideCondition(a), ChangeSideCondition(b)) => a == b,
            (ChangeVolatileStatusDuration(a), ChangeVolatileStatusDuration(b)) => a == b,
            (ChangeWeather(a), ChangeWeather(b)) => a == b,
            (DecrementWeatherTurnsRemaining, DecrementWeatherTurnsRemaining) => true,
            (ChangeTerrain(a), ChangeTerrain(b)) => a == b,
            (DecrementTerrainTurnsRemaining, DecrementTerrainTurnsRemaining) => true,
            (ChangeType(a), ChangeType(b)) => a == b,
            (ChangeAbility(a), ChangeAbility(b)) => a == b,
            (ChangeBaseAbility(a), ChangeBaseAbility(b)) => a == b,
            (ChangeItem(a), ChangeItem(b)) => a == b,
            (ChangeAttack(a), ChangeAttack(b)) => a == b,
            (ChangeDefense(a), ChangeDefense(b)) => a == b,
            (ChangeSpecialAttack(a), ChangeSpecialAttack(b)) => a == b,
            (ChangeSpecialDefense(a), ChangeSpecialDefense(b)) => a == b,
            (ChangeSpeed(a), ChangeSpeed(b)) => a == b,
            (DisableMove(a), DisableMove(b)) => a == b,
            (EnableMove(a), EnableMove(b)) => a == b,
            (ChangeWish(a), ChangeWish(b)) => a == b,
            (DecrementWish(a), DecrementWish(b)) => a == b,
            (SetFutureSight(a), SetFutureSight(b)) => a == b,
            (DecrementFutureSight(a), DecrementFutureSight(b)) => a == b,
            (DamageSubstitute(a), DamageSubstitute(b)) => a == b,
            (DecrementRestTurns(a), DecrementRestTurns(b)) => a == b,
            (SetRestTurns(a), SetRestTurns(b)) => a == b,
            (SetSleepTurns(a), SetSleepTurns(b)) => a == b,
            (SetFreezeTurns(a), SetFreezeTurns(b)) => a == b,
            (ChangeSubstituteHealth(a), ChangeSubstituteHealth(b)) => a == b,
            (FormeChange(a), FormeChange(b)) => a == b,
            (SetSideOneMoveSecondSwitchOutMove(a), SetSideOneMoveSecondSwitchOutMove(b)) => a == b,
            (SetSideTwoMoveSecondSwitchOutMove(a), SetSideTwoMoveSecondSwitchOutMove(b)) => a == b,
            (ToggleBatonPassing(a), ToggleBatonPassing(b)) => a == b,
            (ToggleShedTailing(a), ToggleShedTailing(b)) => a == b,
            (SetLastUsedMove(a), SetLastUsedMove(b)) => a == b,
            (ChangeDamageDealtDamage(a), ChangeDamageDealtDamage(b)) => a == b,
            (ChangeDamageDealtMoveCatagory(a), ChangeDamageDealtMoveCatagory(b)) => a == b,
            (ToggleDamageDealtHitSubstitute(a), ToggleDamageDealtHitSubstitute(b)) => a == b,
            (DecrementPP(a), DecrementPP(b)) => a == b,
            (ToggleTrickRoom(a), ToggleTrickRoom(b)) => a == b,
            (DecrementTrickRoomTurnsRemaining, DecrementTrickRoomTurnsRemaining) => true,
            (ToggleSideOneForceSwitch, ToggleSideOneForceSwitch) => true,
            (ToggleSideTwoForceSwitch, ToggleSideTwoForceSwitch) => true,
            (ToggleTerastallized(a), ToggleTerastallized(b)) => a == b,
            (ToggleMegaEvolved(a), ToggleMegaEvolved(b)) => a == b,
            (ToggleSwordBoostUsed(a), ToggleSwordBoostUsed(b)) => a == b,
            (ToggleShieldBoostUsed(a), ToggleShieldBoostUsed(b)) => a == b,
            (TeamPreview(a), TeamPreview(b)) => a == b,
            _ => false,
        }
    }
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Instruction::Switch(s) => {
                write!(
                    f,
                    "Switch {:?}: {:?} -> {:?}",
                    s.side_ref, s.previous_index, s.next_index
                )
            }
            Instruction::ApplyVolatileStatus(a) => {
                write!(
                    f,
                    "ApplyVolatileStatus {:?}: {:?}",
                    a.side_ref, a.volatile_status
                )
            }
            Instruction::RemoveVolatileStatus(r) => {
                write!(
                    f,
                    "RemoveVolatileStatus {:?}: {:?}",
                    r.side_ref, r.volatile_status
                )
            }
            Instruction::ChangeStatus(c) => {
                write!(
                    f,
                    "ChangeStatus {:?}-{:?}: {:?} -> {:?}",
                    c.side_ref, c.pokemon_index, c.old_status, c.new_status
                )
            }
            Instruction::Heal(h) => {
                write!(f, "Heal {:?}: {:?}", h.side_ref, h.heal_amount)
            }
            Instruction::Damage(d) => {
                write!(f, "Damage {:?}: {}", d.side_ref, d.damage_amount)
            }
            Instruction::DamageWithFaintContext(d) => {
                write!(
                    f,
                    "Damage {:?}: {} ({:?})",
                    d.side_ref, d.damage_amount, d.faint_context
                )
            }
            Instruction::Boost(b) => {
                write!(f, "Boost {:?} {:?}: {:?}", b.side_ref, b.stat, b.amount)
            }
            Instruction::ChangeSideCondition(c) => {
                write!(
                    f,
                    "ChangeSideCondition {:?} {:?}: {:?}",
                    c.side_ref, c.side_condition, c.amount
                )
            }
            Instruction::ChangeVolatileStatusDuration(c) => {
                write!(
                    f,
                    "ChangeVolatileStatusDuration {:?} {:?}: {:?}",
                    c.side_ref, c.volatile_status, c.amount
                )
            }
            Instruction::ChangeWeather(c) => {
                write!(
                    f,
                    "ChangeWeather: {:?},{:?} -> {:?},{:?}",
                    c.previous_weather,
                    c.previous_weather_turns_remaining,
                    c.new_weather,
                    c.new_weather_turns_remaining
                )
            }
            Instruction::DecrementWeatherTurnsRemaining => {
                write!(f, "DecrementWeatherTurnsRemaining",)
            }
            Instruction::ChangeTerrain(c) => {
                write!(
                    f,
                    "ChangeTerrain: {:?},{:?} -> {:?},{:?}",
                    c.previous_terrain,
                    c.previous_terrain_turns_remaining,
                    c.new_terrain,
                    c.new_terrain_turns_remaining
                )
            }
            Instruction::DecrementTerrainTurnsRemaining => {
                write!(f, "DecrementTerrainTurnsRemaining",)
            }
            Instruction::ChangeType(c) => {
                write!(
                    f,
                    "ChangeType {:?}: {:?} -> {:?}",
                    c.side_ref, c.old_types, c.new_types
                )
            }
            Instruction::ChangeAbility(c) => {
                write!(f, "ChangeAbility {:?}: {:?}", c.side_ref, c.ability_change)
            }
            Instruction::ChangeBaseAbility(c) => {
                write!(
                    f,
                    "ChangeBaseAbility {:?}: {:?}",
                    c.side_ref, c.ability_change
                )
            }
            Instruction::ChangeItem(c) => {
                write!(
                    f,
                    "ChangeItem {:?}: {:?} -> {:?}",
                    c.side_ref, c.current_item, c.new_item
                )
            }
            Instruction::ChangeAttack(c) => {
                write!(f, "ChangeAttack {:?}: {}", c.side_ref, c.amount)
            }
            Instruction::ChangeDefense(c) => {
                write!(f, "ChangeDefense {:?}: {}", c.side_ref, c.amount)
            }
            Instruction::ChangeSpecialAttack(c) => {
                write!(f, "ChangeSpecialAttack {:?}: {}", c.side_ref, c.amount)
            }
            Instruction::ChangeSpecialDefense(c) => {
                write!(f, "ChangeSpecialDefense {:?}: {}", c.side_ref, c.amount)
            }
            Instruction::ChangeSpeed(c) => {
                write!(f, "ChangeSpeed {:?}: {}", c.side_ref, c.amount)
            }
            Instruction::DisableMove(d) => {
                write!(f, "DisableMove {:?}: {:?}", d.side_ref, d.move_index)
            }
            Instruction::EnableMove(e) => {
                write!(f, "EnableMove {:?}: {:?}", e.side_ref, e.move_index)
            }
            Instruction::ChangeWish(s) => {
                write!(f, "SetWish {:?}: {:?}", s.side_ref, s.wish_amount_change)
            }
            Instruction::DecrementWish(d) => {
                write!(f, "DecrementWish {:?}", d.side_ref)
            }
            Instruction::SetFutureSight(s) => {
                write!(
                    f,
                    "SetFutureSight {:?}: {:?} -> {:?}",
                    s.side_ref, s.previous_pokemon_index, s.pokemon_index
                )
            }
            Instruction::DecrementFutureSight(d) => {
                write!(f, "DecrementFutureSight {:?}", d.side_ref)
            }
            Instruction::DamageSubstitute(d) => {
                write!(
                    f,
                    "DamageSubstitute {:?}: {:?}",
                    d.side_ref, d.damage_amount
                )
            }
            Instruction::DecrementRestTurns(d) => {
                write!(f, "DecrementRestTurns {:?}", d.side_ref)
            }
            Instruction::SetRestTurns(s) => {
                write!(
                    f,
                    "SetRestTurns {:?}-{:?}: {:?} -> {:?}",
                    s.side_ref, s.pokemon_index, s.previous_turns, s.new_turns
                )
            }
            Instruction::SetSleepTurns(s) => {
                write!(
                    f,
                    "SetSleepTurns {:?}-{:?}: {:?} -> {:?}",
                    s.side_ref, s.pokemon_index, s.previous_turns, s.new_turns
                )
            }
            Instruction::SetFreezeTurns(s) => {
                write!(
                    f,
                    "SetFreezeTurns {:?}-{:?}: {:?} -> {:?}",
                    s.side_ref, s.pokemon_index, s.previous_turns, s.new_turns
                )
            }
            Instruction::ChangeSubstituteHealth(s) => {
                write!(
                    f,
                    "ChangeSubstituteHealth {:?}: {:?}",
                    s.side_ref, s.health_change,
                )
            }
            Instruction::FormeChange(s) => {
                write!(f, "FormeChange {:?} {}", s.side_ref, s.name_change)
            }
            Instruction::SetSideOneMoveSecondSwitchOutMove(s) => {
                write!(
                    f,
                    "SideOneMoveSecondSwitchOutMove: {:?} -> {:?}",
                    s.previous_choice, s.new_choice
                )
            }
            Instruction::SetSideTwoMoveSecondSwitchOutMove(s) => {
                write!(
                    f,
                    "SideTwoMoveSecondSwitchOutMove: {:?} -> {:?}",
                    s.previous_choice, s.new_choice
                )
            }
            Instruction::ToggleBatonPassing(s) => {
                write!(f, "ToggleBatonPassing {:?}", s.side_ref)
            }
            Instruction::ToggleShedTailing(s) => {
                write!(f, "ToggleShedTailing {:?}", s.side_ref)
            }
            Instruction::ToggleTerastallized(s) => {
                write!(f, "ToggleTerastallized {:?}", s.side_ref)
            }
            Instruction::ToggleMegaEvolved(s) => {
                write!(
                    f,
                    "ToggleMegaEvolved {:?}: {:?}",
                    s.side_ref, s.pokemon_index
                )
            }
            Instruction::ToggleSwordBoostUsed(s) => {
                write!(
                    f,
                    "ToggleSwordBoostUsed {:?}: {:?}",
                    s.side_ref, s.pokemon_index
                )
            }
            Instruction::ToggleShieldBoostUsed(s) => {
                write!(
                    f,
                    "ToggleShieldBoostUsed {:?}: {:?}",
                    s.side_ref, s.pokemon_index
                )
            }
            Instruction::SetLastUsedMove(s) => {
                write!(
                    f,
                    "SetLastUsedMove {:?}: {:?} -> {:?}",
                    s.side_ref, s.previous_last_used_move, s.last_used_move
                )
            }
            Instruction::ChangeDamageDealtDamage(s) => {
                write!(
                    f,
                    "ChangeDamageDealtDamage {:?}: {:?}",
                    s.side_ref, s.damage_change
                )
            }
            Instruction::ChangeDamageDealtMoveCatagory(s) => {
                write!(
                    f,
                    "ChangeDamageDealtMoveCatagory {:?}: {:?} -> {:?}",
                    s.side_ref, s.previous_move_category, s.move_category
                )
            }
            Instruction::ToggleDamageDealtHitSubstitute(s) => {
                write!(f, "ToggleDamageDealtHitSubstitute {:?}", s.side_ref)
            }
            Instruction::DecrementPP(s) => {
                write!(
                    f,
                    "DecrementPP {:?}: {:?} {}",
                    s.side_ref, s.move_index, s.amount
                )
            }
            Instruction::ToggleTrickRoom(i) => {
                write!(
                    f,
                    "ToggleTrickRoom: {:?},{:?} -> {:?},{:?}",
                    i.currently_active,
                    i.previous_trickroom_turns_remaining,
                    !i.currently_active,
                    i.new_trickroom_turns_remaining,
                )
            }
            Instruction::DecrementTrickRoomTurnsRemaining => {
                write!(f, "DecrementTrickRoomTurnsRemaining")
            }
            Instruction::ToggleSideOneForceSwitch => {
                write!(f, "ToggleSideOneForceSwitch")
            }
            Instruction::ToggleSideTwoForceSwitch => {
                write!(f, "ToggleSideTwoForceSwitch")
            }
            Instruction::TeamPreview(s) => {
                write!(
                    f,
                    "TeamPreview {:?}: {:?},{:?},{:?}",
                    s.side_ref, s.lead_index, s.reserve_index_one, s.reserve_index_two
                )
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ChangeDamageDealtDamageInstruction {
    pub side_ref: SideReference,
    pub damage_change: i16,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ChangeDamageDealtMoveCategoryInstruction {
    pub side_ref: SideReference,
    pub move_category: MoveCategory,
    pub previous_move_category: MoveCategory,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ToggleDamageDealtHitSubstituteInstruction {
    pub side_ref: SideReference,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DecrementPPInstruction {
    pub side_ref: SideReference,
    pub move_index: PokemonMoveIndex,
    pub amount: i8,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SetLastUsedMoveInstruction {
    pub side_ref: SideReference,
    pub last_used_move: LastUsedMove,
    pub previous_last_used_move: LastUsedMove,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ToggleBatonPassingInstruction {
    pub side_ref: SideReference,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ToggleShedTailingInstruction {
    pub side_ref: SideReference,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DecrementRestTurnsInstruction {
    pub side_ref: SideReference,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SetSleepTurnsInstruction {
    pub side_ref: SideReference,
    pub pokemon_index: PokemonIndex,
    pub new_turns: i8,
    pub previous_turns: i8,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SetSecondMoveSwitchOutMoveInstruction {
    pub new_choice: Choices,
    pub previous_choice: Choices,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ChangeWishInstruction {
    pub side_ref: SideReference,
    pub wish_amount_change: i16,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DecrementWishInstruction {
    pub side_ref: SideReference,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SetFutureSightInstruction {
    pub side_ref: SideReference,
    pub pokemon_index: PokemonIndex,
    pub previous_pokemon_index: PokemonIndex,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DecrementFutureSightInstruction {
    pub side_ref: SideReference,
}

#[derive(Debug, PartialEq, Clone)]
pub struct EnableMoveInstruction {
    pub side_ref: SideReference,
    pub move_index: PokemonMoveIndex,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DisableMoveInstruction {
    pub side_ref: SideReference,
    pub move_index: PokemonMoveIndex,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ChangeItemInstruction {
    pub side_ref: SideReference,
    pub current_item: Items,
    pub new_item: Items,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ChangeStatInstruction {
    pub side_ref: SideReference,
    pub amount: i16,
}

#[derive(Debug, PartialEq, Clone)]
pub struct HealInstruction {
    pub side_ref: SideReference,
    pub heal_amount: i16,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DamageInstruction {
    pub side_ref: SideReference,
    pub damage_amount: i16,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum FaintCause {
    Unknown,
    DirectMove,
    DestinyBond,
    Recoil,
    SelfKoMove,
    PerishSong,
    Residual,
    Ability,
    Item,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum FaintEffect {
    None,
    Move(Choices),
    Ability(Abilities),
    Item(Items),
    Status(PokemonStatus),
    Weather(Weather),
    Volatile(PokemonVolatileStatus),
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct FaintContext {
    pub source_side: Option<SideReference>,
    pub source_index: Option<PokemonIndex>,
    pub cause: FaintCause,
    pub effect: FaintEffect,
}

impl FaintContext {
    pub fn unknown() -> FaintContext {
        FaintContext {
            source_side: None,
            source_index: None,
            cause: FaintCause::Unknown,
            effect: FaintEffect::None,
        }
    }

    pub fn move_effect(
        source_side: SideReference,
        source_index: PokemonIndex,
        cause: FaintCause,
        move_id: Choices,
    ) -> FaintContext {
        FaintContext {
            source_side: Some(source_side),
            source_index: Some(source_index),
            cause,
            effect: FaintEffect::Move(move_id),
        }
    }

    pub fn residual(effect: FaintEffect) -> FaintContext {
        FaintContext {
            source_side: None,
            source_index: None,
            cause: FaintCause::Residual,
            effect,
        }
    }

    pub fn ability_effect(
        source_side: SideReference,
        source_index: PokemonIndex,
        ability: Abilities,
    ) -> FaintContext {
        FaintContext {
            source_side: Some(source_side),
            source_index: Some(source_index),
            cause: FaintCause::Ability,
            effect: FaintEffect::Ability(ability),
        }
    }

    pub fn item_effect(
        source_side: SideReference,
        source_index: PokemonIndex,
        item: Items,
    ) -> FaintContext {
        FaintContext {
            source_side: Some(source_side),
            source_index: Some(source_index),
            cause: FaintCause::Item,
            effect: FaintEffect::Item(item),
        }
    }
}

impl Default for FaintContext {
    fn default() -> FaintContext {
        FaintContext::unknown()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct DamageWithFaintContextInstruction {
    pub side_ref: SideReference,
    pub damage_amount: i16,
    pub faint_context: FaintContext,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct FaintEvent {
    pub target_side: SideReference,
    pub target_index: PokemonIndex,
    pub source_side: Option<SideReference>,
    pub source_index: Option<PokemonIndex>,
    pub cause: FaintCause,
    pub effect: FaintEffect,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ChangeSubsituteHealthInstruction {
    pub side_ref: SideReference,
    pub health_change: i16,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FormeChangeInstruction {
    pub side_ref: SideReference,

    // PokemonName is represented as i16
    // This is the amount the name has changed by
    pub name_change: i16,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SwitchInstruction {
    pub side_ref: SideReference,
    pub previous_index: PokemonIndex,
    pub next_index: PokemonIndex,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TeamPreviewInstruction {
    pub side_ref: SideReference,
    pub previous_active_index: PokemonIndex,
    pub previous_pokemon: String,
    pub lead_index: PokemonIndex,
    pub reserve_index_one: PokemonIndex,
    pub reserve_index_two: PokemonIndex,
}

// pokemon_index is present because even reserve pokemon can have their status
// changed (i.e. healbell)
#[derive(Debug, PartialEq, Clone)]
pub struct ChangeStatusInstruction {
    pub side_ref: SideReference,
    pub pokemon_index: PokemonIndex,
    pub old_status: PokemonStatus,
    pub new_status: PokemonStatus,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ApplyVolatileStatusInstruction {
    pub side_ref: SideReference,
    pub volatile_status: PokemonVolatileStatus,
}

#[derive(Debug, PartialEq, Clone)]
pub struct RemoveVolatileStatusInstruction {
    pub side_ref: SideReference,
    pub volatile_status: PokemonVolatileStatus,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BoostInstruction {
    pub side_ref: SideReference,
    pub stat: PokemonBoostableStat,
    pub amount: i8,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ChangeSideConditionInstruction {
    pub side_ref: SideReference,
    pub side_condition: PokemonSideCondition,
    pub amount: i8,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ChangeVolatileStatusDurationInstruction {
    pub side_ref: SideReference,
    pub volatile_status: PokemonVolatileStatus,
    pub amount: i8,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ChangeWeather {
    pub new_weather: Weather,
    pub new_weather_turns_remaining: i8,
    pub previous_weather: Weather,
    pub previous_weather_turns_remaining: i8,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ChangeTerrain {
    pub new_terrain: Terrain,
    pub new_terrain_turns_remaining: i8,
    pub previous_terrain: Terrain,
    pub previous_terrain_turns_remaining: i8,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ToggleTrickRoomInstruction {
    pub currently_active: bool,
    pub new_trickroom_turns_remaining: i8,
    pub previous_trickroom_turns_remaining: i8,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ToggleTerastallizedInstruction {
    pub side_ref: SideReference,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ToggleMegaEvolvedInstruction {
    pub side_ref: SideReference,
    pub pokemon_index: PokemonIndex,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ToggleAbilityOnStartFlagInstruction {
    pub side_ref: SideReference,
    pub pokemon_index: PokemonIndex,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ChangeType {
    pub side_ref: SideReference,
    pub new_types: (PokemonType, PokemonType),
    pub old_types: (PokemonType, PokemonType),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ChangeAbilityInstruction {
    pub side_ref: SideReference,

    // Abilities enum is an i16
    // This is the amount the ability has changed by
    pub ability_change: i16,
}

#[cfg(test)]
mod test {
    use super::Instruction;

    // Make sure that the size of the Instruction enum doesn't change
    #[test]
    fn test_instruction_size() {
        assert_eq!(size_of::<Instruction>(), 32);
        assert_eq!(align_of::<Instruction>(), 8);
    }
}
