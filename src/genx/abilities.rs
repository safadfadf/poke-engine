#![allow(unused_variables)]
use super::damage_calc::type_effectiveness_modifier;
use super::generate_instructions::{
    add_remove_status_instructions, apply_boost_instruction_with_effective, immune_to_status,
};
use super::items::{
    consume_berry, get_choice_move_disable_instructions, opponent_blocks_berries, Items,
};
use super::state::{PokemonVolatileStatus, Terrain, Weather};
use crate::choices::{
    Boost, Choice, Choices, Effect, Heal, MoveCategory, MoveTarget, Secondary, StatBoosts,
    VolatileStatus,
};
use crate::define_enum_with_from_str;
#[cfg(feature = "gen9")]
use crate::instruction::ToggleAbilityOnStartFlagInstruction;
use crate::instruction::{
    ApplyVolatileStatusInstruction, BoostInstruction, ChangeAbilityInstruction,
    ChangeItemInstruction, ChangeSideConditionInstruction, ChangeStatusInstruction, ChangeTerrain,
    ChangeType, ChangeVolatileStatusDurationInstruction, ChangeWeather,
    DamageWithFaintContextInstruction, FaintContext, FormeChangeInstruction, HealInstruction,
    Instruction, StateInstructions,
};
use crate::pokemon::PokemonName;
use crate::state::{
    PokemonBoostableStat, PokemonSideCondition, PokemonStatus, PokemonType, Side, SideReference,
    State,
};
use std::cmp;

#[cfg(any(feature = "gen3", feature = "gen4", feature = "gen5"))]
pub const WEATHER_ABILITY_TURNS: i8 = -1;

#[cfg(any(feature = "gen6", feature = "gen7", feature = "gen8", feature = "gen9"))]
pub const WEATHER_ABILITY_TURNS: i8 = 5;

fn active_strong_weather_blocks_regular_weather(state: &State) -> bool {
    state.active_strong_weather_blocks_regular_weather()
}

fn regular_weather_should_be_set(state: &State, weather: Weather) -> bool {
    !active_strong_weather_blocks_regular_weather(state) && state.weather_should_be_set(weather)
}

fn weather_should_be_set(state: &State, weather: Weather) -> bool {
    state.weather_should_be_set(weather)
}

fn terrain_should_be_set(state: &State, terrain: Terrain) -> bool {
    state.terrain_should_be_set(terrain)
}

define_enum_with_from_str! {
    #[repr(i16)]
    #[derive(PartialEq, Debug, Clone, Copy)]
    Abilities {
        NONE,
        ARMORTAIL,
        RIPEN,
        TANGLEDFEET,
        DRAGONSMAW,
        CLEARBODY,
        GALVANIZE,
        VITALSPIRIT,
        AERILATE,
        DEFIANT,
        CUTECHARM,
        NEUROFORCE,
        SOUNDPROOF,
        RKSSYSTEM,
        POISONPOINT,
        STAKEOUT,
        UNNERVE,
        ROCKHEAD,
        AURABREAK,
        MIMICRY,
        BULLETPROOF,
        POWEROFALCHEMY,
        TECHNICIAN,
        MULTISCALE,
        ARENATRAP,
        BATTLEBOND,
        DISGUISE,
        EARLYBIRD,
        LIGHTNINGROD,
        MAGICIAN,
        REFRIGERATE,
        FRIENDGUARD,
        NOABILITY,
        GULPMISSILE,
        POWERCONSTRUCT,
        FORECAST,
        PRANKSTER,
        PROTEAN,
        ASONEGLASTRIER,
        SHADOWTAG,
        SHARPNESS,
        WINDRIDER,
        SKILLLINK,
        INTREPIDSWORD,
        SOULHEART,
        SWIFTSWIM,
        EARTHEATER,
        SUPERLUCK,
        SUPREMEOVERLORD,
        INSOMNIA,
        DANCER,
        STEAMENGINE,
        ANGERPOINT,
        CONTRARY,
        MAGMAARMOR,
        HUNGERSWITCH,
        RECEIVER,
        ZENMODE,
        EMERGENCYEXIT,
        ILLUSION,
        WEAKARMOR,
        DROUGHT,
        INNARDSOUT,
        SHIELDSDOWN,
        ADAPTABILITY,
        CORROSION,
        LONGREACH,
        PUREPOWER,
        TINTEDLENS,
        QUEENLYMAJESTY,
        DESOLATELAND,
        MOXIE,
        SAPSIPPER,
        SLUSHRUSH,
        BIGPECKS,
        STALL,
        WHITESMOKE,
        FLAREBOOST,
        SHADOWSHIELD,
        LIQUIDVOICE,
        MISTYSURGE,
        MULTITYPE,
        NOGUARD,
        TORRENT,
        DELTASTREAM,
        KLUTZ,
        LIBERO,
        SERENEGRACE,
        CURSEDBODY,
        UNAWARE,
        LIGHTMETAL,
        MARVELSCALE,
        TELEPATHY,
        QUICKDRAW,
        HYPERCUTTER,
        SYMBIOSIS,
        PLUS,
        MIRRORARMOR,
        PASTELVEIL,
        TOUGHCLAWS,
        EFFECTSPORE,
        MUMMY,
        BADDREAMS,
        MAGICGUARD,
        SANDSTREAM,
        POWERSPOT,
        FLAMEBODY,
        RECKLESS,
        PRESSURE,
        GOOEY,
        IMMUNITY,
        LEAFGUARD,
        HUGEPOWER,
        SOLARPOWER,
        SCHOOLING,
        MOTORDRIVE,
        ANTICIPATION,
        MERCILESS,
        TRACE,
        NATURALCURE,
        HARVEST,
        SUCTIONCUPS,
        ICEFACE,
        ROUGHSKIN,
        WONDERGUARD,
        WATERVEIL,
        FAIRYAURA,
        SANDSPIT,
        SEEDSOWER,
        TOXICDEBRIS,
        INTIMIDATE,
        DAUNTLESSSHIELD,
        AROMAVEIL,
        AIRLOCK,
        NORMALIZE,
        DARKAURA,
        VICTORYSTAR,
        GRASSYSURGE,
        STURDY,
        PICKPOCKET,
        ELECTRICSURGE,
        HADRONENGINE,
        ORICHALCUMPULSE,
        RUNAWAY,
        OBLIVIOUS,
        SURGESURFER,
        LEVITATE,
        ASONESPECTRIER,
        PICKUP,
        ICEBODY,
        CURIOUSMEDICINE,
        FLOWERVEIL,
        STATIC,
        WONDERSKIN,
        OVERGROW,
        PROPELLERTAIL,
        THICKFAT,
        GLUTTONY,
        KEENEYE,
        MOUNTAINEER,
        FLASHFIRE,
        COMPOUNDEYES,
        STEELWORKER,
        COMATOSE,
        BALLFETCH,
        DAZZLING,
        DOWNLOAD,
        TRANSISTOR,
        MOLDBREAKER,
        MYCELIUMMIGHT,
        LIQUIDOOZE,
        POISONHEAL,
        PRISMARMOR,
        SNIPER,
        STENCH,
        COMPETITIVE,
        SWARM,
        STALWART,
        ILLUMINATE,
        TURBOBLAZE,
        GORILLATACTICS,
        SPEEDBOOST,
        GUARDDOG,
        HEATPROOF,
        SNOWCLOAK,
        TERAVOLT,
        CHILLINGNEIGH,
        SHIELDDUST,
        RIVALRY,
        PRIMORDIALSEA,
        SCREENCLEANER,
        MAGNETPULL,
        HONEYGATHER,
        COTTONDOWN,
        GRASSPELT,
        BATTLEARMOR,
        BEASTBOOST,
        BERSERK,
        MINUS,
        RAINDISH,
        SYNCHRONIZE,
        FILTER,
        TRUANT,
        FURCOAT,
        FULLMETALBODY,
        REGENERATOR,
        FOREWARN,
        IRONBARBS,
        STAMINA,
        SANDRUSH,
        COLORCHANGE,
        BLAZE,
        ANALYTIC,
        TANGLINGHAIR,
        CLOUDNINE,
        STEELYSPIRIT,
        QUICKFEET,
        MAGICBOUNCE,
        MEGALAUNCHER,
        HEAVYMETAL,
        STORMDRAIN,
        PIXILATE,
        WATERCOMPACTION,
        JUSTIFIED,
        SLOWSTART,
        SNOWWARNING,
        FLOWERGIFT,
        SHEDSKIN,
        WIMPOUT,
        ICESCALES,
        INFILTRATOR,
        LIMBER,
        PSYCHICSURGE,
        DEFEATIST,
        WATERABSORB,
        IMPOSTER,
        DRYSKIN,
        FLUFFY,
        UNBURDEN,
        CHEEKPOUCH,
        STANCECHANGE,
        MOODY,
        ROCKYPAYLOAD,
        PUNKROCK,
        SANDVEIL,
        PARENTALBOND,
        STRONGJAW,
        BATTERY,
        HEALER,
        STEADFAST,
        DAMP,
        PERISHBODY,
        TRIAGE,
        SHEERFORCE,
        OWNTEMPO,
        FRISK,
        VOLTABSORB,
        GALEWINGS,
        AFTERMATH,
        STICKYHOLD,
        GRIMNEIGH,
        IRONFIST,
        REBOUND,
        UNSEENFIST,
        SOLIDROCK,
        HUSTLE,
        HYDRATION,
        SCRAPPY,
        MINDSEYE,
        OVERCOAT,
        NEUTRALIZINGGAS,
        SWEETVEIL,
        DRIZZLE,
        INNERFOCUS,
        POISONTOUCH,
        WANDERINGSPIRIT,
        GUTS,
        SHELLARMOR,
        RATTLED,
        WATERBUBBLE,
        SANDFORCE,
        TOXICBOOST,
        PERSISTENT,
        CHLOROPHYLL,
        SIMPLE,
        PURIFYINGSALT,
        EMBODYASPECTWELLSPRING,
        EMBODYASPECTCORNERSTONE,
        EMBODYASPECTHEARTHFLAME,
        EMBODYASPECTTEAL,
        ANGERSHELL,
        BEADSOFRUIN,
        COMMANDER,
        COSTAR,
        CUDCHEW,
        ELECTROMORPHOSIS,
        EMBODYASPECT,
        GOODASGOLD,
        HOSPITALITY,
        LINGERINGAROMA,
        OPPORTUNIST,
        POISONPUPPETEER,
        PROTOSYNTHESIS,
        QUARKDRIVE,
        SUPERSWEETSYRUP,
        SWORDOFRUIN,
        TABLETSOFRUIN,
        TERASHELL,
        TERASHIFT,
        TERAFORMZERO,
        THERMALEXCHANGE,
        TOXICCHAIN,
        VESSELOFRUIN,
        WELLBAKEDBODY,
        WINDPOWER,
        ZEROTOHERO,
        MEGASOL,
        DRAGONIZE,
        SPICYSPRAY,
        PIERCINGDRILL,
    },
    default = NONE
}

// https://bulbapedia.bulbagarden.net/wiki/Ignoring_Abilities#Ignorable_Abilities
pub(crate) fn mold_breaker_ignores(ability: &Abilities) -> bool {
    match ability {
        Abilities::BATTLEARMOR
        | Abilities::CLEARBODY
        | Abilities::ARMORTAIL
        | Abilities::EARTHEATER
        | Abilities::GUARDDOG
        | Abilities::GOODASGOLD
        | Abilities::ILLUMINATE
        | Abilities::MINDSEYE
        | Abilities::PURIFYINGSALT
        | Abilities::TERASHELL
        | Abilities::TABLETSOFRUIN
        | Abilities::THERMALEXCHANGE
        | Abilities::WELLBAKEDBODY
        | Abilities::VESSELOFRUIN
        | Abilities::DAMP
        | Abilities::DRYSKIN
        | Abilities::FILTER
        | Abilities::FLASHFIRE
        | Abilities::FLOWERGIFT
        | Abilities::HEATPROOF
        | Abilities::HYPERCUTTER
        | Abilities::IMMUNITY
        | Abilities::INNERFOCUS
        | Abilities::INSOMNIA
        | Abilities::KEENEYE
        | Abilities::LEAFGUARD
        | Abilities::LEVITATE
        | Abilities::LIGHTNINGROD
        | Abilities::LIMBER
        | Abilities::MAGMAARMOR
        | Abilities::MARVELSCALE
        | Abilities::MOTORDRIVE
        | Abilities::WINDRIDER
        | Abilities::OBLIVIOUS
        | Abilities::OWNTEMPO
        | Abilities::SANDVEIL
        | Abilities::SHELLARMOR
        | Abilities::SHIELDDUST
        | Abilities::SIMPLE
        | Abilities::SNOWCLOAK
        | Abilities::SOLIDROCK
        | Abilities::SOUNDPROOF
        | Abilities::STICKYHOLD
        | Abilities::STORMDRAIN
        | Abilities::STURDY
        | Abilities::SUCTIONCUPS
        | Abilities::TANGLEDFEET
        | Abilities::THICKFAT
        | Abilities::UNAWARE
        | Abilities::VITALSPIRIT
        | Abilities::VOLTABSORB
        | Abilities::WATERABSORB
        | Abilities::WATERVEIL
        | Abilities::WHITESMOKE
        | Abilities::WONDERGUARD
        | Abilities::BIGPECKS
        | Abilities::CONTRARY
        | Abilities::FRIENDGUARD
        | Abilities::HEAVYMETAL
        | Abilities::LIGHTMETAL
        | Abilities::MAGICBOUNCE
        | Abilities::MULTISCALE
        | Abilities::SAPSIPPER
        | Abilities::TELEPATHY
        | Abilities::WONDERSKIN
        | Abilities::AURABREAK
        | Abilities::AROMAVEIL
        | Abilities::BULLETPROOF
        | Abilities::FLOWERVEIL
        | Abilities::FURCOAT
        | Abilities::OVERCOAT
        | Abilities::SWEETVEIL
        | Abilities::DAZZLING
        | Abilities::DISGUISE
        | Abilities::FLUFFY
        | Abilities::QUEENLYMAJESTY
        | Abilities::WATERBUBBLE
        | Abilities::ICESCALES
        | Abilities::ICEFACE
        | Abilities::MIRRORARMOR
        | Abilities::PASTELVEIL
        | Abilities::PUNKROCK
        | Abilities::FAIRYAURA
        | Abilities::DARKAURA => true,
        _ => false,
    }
}

fn ability_cannot_be_replaced_by_contact(ability: Abilities) -> bool {
    matches!(
        ability,
        Abilities::ASONEGLASTRIER
            | Abilities::ASONESPECTRIER
            | Abilities::BATTLEBOND
            | Abilities::COMATOSE
            | Abilities::DISGUISE
            | Abilities::GULPMISSILE
            | Abilities::ICEFACE
            | Abilities::MULTITYPE
            | Abilities::POWERCONSTRUCT
            | Abilities::RKSSYSTEM
            | Abilities::SCHOOLING
            | Abilities::SHIELDSDOWN
            | Abilities::STANCECHANGE
            | Abilities::TERASHIFT
            | Abilities::ZENMODE
            | Abilities::ZEROTOHERO
    )
}

fn ability_fails_skill_swap(ability: Abilities) -> bool {
    matches!(
        ability,
        Abilities::ASONEGLASTRIER
            | Abilities::ASONESPECTRIER
            | Abilities::BATTLEBOND
            | Abilities::COMATOSE
            | Abilities::COMMANDER
            | Abilities::DISGUISE
            | Abilities::EMBODYASPECT
            | Abilities::EMBODYASPECTCORNERSTONE
            | Abilities::EMBODYASPECTHEARTHFLAME
            | Abilities::EMBODYASPECTTEAL
            | Abilities::EMBODYASPECTWELLSPRING
            | Abilities::HUNGERSWITCH
            | Abilities::ICEFACE
            | Abilities::ILLUSION
            | Abilities::MULTITYPE
            | Abilities::NEUTRALIZINGGAS
            | Abilities::POISONPUPPETEER
            | Abilities::POWERCONSTRUCT
            | Abilities::PROTOSYNTHESIS
            | Abilities::QUARKDRIVE
            | Abilities::RKSSYSTEM
            | Abilities::SCHOOLING
            | Abilities::SHIELDSDOWN
            | Abilities::STANCECHANGE
            | Abilities::TERAFORMZERO
            | Abilities::TERASHELL
            | Abilities::TERASHIFT
            | Abilities::WONDERGUARD
            | Abilities::ZENMODE
            | Abilities::ZEROTOHERO
    )
}

fn ability_fails_trace(ability: Abilities) -> bool {
    matches!(
        ability,
        Abilities::NONE
            | Abilities::ASONEGLASTRIER
            | Abilities::ASONESPECTRIER
            | Abilities::BATTLEBOND
            | Abilities::COMATOSE
            | Abilities::COMMANDER
            | Abilities::DISGUISE
            | Abilities::EMBODYASPECTCORNERSTONE
            | Abilities::EMBODYASPECTHEARTHFLAME
            | Abilities::EMBODYASPECTTEAL
            | Abilities::EMBODYASPECTWELLSPRING
            | Abilities::FLOWERGIFT
            | Abilities::FORECAST
            | Abilities::HUNGERSWITCH
            | Abilities::ICEFACE
            | Abilities::ILLUSION
            | Abilities::IMPOSTER
            | Abilities::MULTITYPE
            | Abilities::NEUTRALIZINGGAS
            | Abilities::POISONPUPPETEER
            | Abilities::POWERCONSTRUCT
            | Abilities::POWEROFALCHEMY
            | Abilities::PROTOSYNTHESIS
            | Abilities::QUARKDRIVE
            | Abilities::RECEIVER
            | Abilities::RKSSYSTEM
            | Abilities::SCHOOLING
            | Abilities::SHIELDSDOWN
            | Abilities::STANCECHANGE
            | Abilities::TERAFORMZERO
            | Abilities::TERASHELL
            | Abilities::TERASHIFT
            | Abilities::TRACE
            | Abilities::ZENMODE
            | Abilities::ZEROTOHERO
    )
}

fn protosynthesus_or_quarkdrive_on_switch_in(
    thing_is_active: bool,
    item_is_active: bool,
    volatile: PokemonVolatileStatus,
    booster_volatile: PokemonVolatileStatus,
    instructions: &mut StateInstructions,
    attacking_side: &mut Side,
    side_ref: &SideReference,
) {
    let active_pkmn = attacking_side.get_active();
    if thing_is_active {
        if !attacking_side.volatile_statuses.contains(&volatile) {
            instructions
                .instruction_list
                .push(Instruction::ApplyVolatileStatus(
                    ApplyVolatileStatusInstruction {
                        side_ref: *side_ref,
                        volatile_status: volatile,
                    },
                ));
            attacking_side.volatile_statuses.insert(volatile);
        }
    } else if active_pkmn.item == Items::BOOSTERENERGY && item_is_active {
        instructions
            .instruction_list
            .push(Instruction::ChangeItem(ChangeItemInstruction {
                side_ref: *side_ref,
                current_item: Items::BOOSTERENERGY,
                new_item: Items::NONE,
            }));
        instructions
            .instruction_list
            .push(Instruction::ApplyVolatileStatus(
                ApplyVolatileStatusInstruction {
                    side_ref: *side_ref,
                    volatile_status: volatile,
                },
            ));
        instructions
            .instruction_list
            .push(Instruction::ApplyVolatileStatus(
                ApplyVolatileStatusInstruction {
                    side_ref: *side_ref,
                    volatile_status: booster_volatile,
                },
            ));
        active_pkmn.item = Items::NONE;
        attacking_side.volatile_statuses.insert(volatile);
        attacking_side.volatile_statuses.insert(booster_volatile);
    }
}

fn protosynthesis_volatile_from_side(side: &Side) -> PokemonVolatileStatus {
    match side.calculate_highest_stat() {
        PokemonBoostableStat::Attack => PokemonVolatileStatus::PROTOSYNTHESISATK,
        PokemonBoostableStat::Defense => PokemonVolatileStatus::PROTOSYNTHESISDEF,
        PokemonBoostableStat::SpecialAttack => PokemonVolatileStatus::PROTOSYNTHESISSPA,
        PokemonBoostableStat::SpecialDefense => PokemonVolatileStatus::PROTOSYNTHESISSPD,
        PokemonBoostableStat::Speed => PokemonVolatileStatus::PROTOSYNTHESISSPE,
        _ => panic!("Invalid stat for protosynthesis"),
    }
}

fn quarkdrive_volatile_from_side(side: &Side) -> PokemonVolatileStatus {
    match side.calculate_highest_stat() {
        PokemonBoostableStat::Attack => PokemonVolatileStatus::QUARKDRIVEATK,
        PokemonBoostableStat::Defense => PokemonVolatileStatus::QUARKDRIVEDEF,
        PokemonBoostableStat::SpecialAttack => PokemonVolatileStatus::QUARKDRIVESPA,
        PokemonBoostableStat::SpecialDefense => PokemonVolatileStatus::QUARKDRIVESPD,
        PokemonBoostableStat::Speed => PokemonVolatileStatus::QUARKDRIVESPE,
        _ => panic!("Invalid stat for quarkdrive"),
    }
}

fn apply_charge_volatile(
    side: &mut Side,
    side_ref: SideReference,
    instructions: &mut StateInstructions,
) {
    if side.get_active_immutable().hp == 0
        || side
            .volatile_statuses
            .contains(&PokemonVolatileStatus::CHARGE)
    {
        return;
    }
    instructions
        .instruction_list
        .push(Instruction::ApplyVolatileStatus(
            ApplyVolatileStatusInstruction {
                side_ref,
                volatile_status: PokemonVolatileStatus::CHARGE,
            },
        ));
    side.volatile_statuses.insert(PokemonVolatileStatus::CHARGE);
}

pub fn ability_before_move(
    state: &mut State,
    choice: &mut Choice,
    side_ref: &SideReference,
    instructions: &mut StateInstructions,
) {
    let defending_side_ref = side_ref.get_other_side();
    let active_ability_is_suppressed = state.active_ability_is_suppressed(side_ref);
    let defending_ability_is_suppressed = state.active_ability_is_suppressed(&defending_side_ref);
    let raw_defending_ability = state
        .get_side_immutable(&defending_side_ref)
        .get_active_immutable()
        .ability;
    let defending_ability = if defending_ability_is_suppressed {
        Abilities::NONE
    } else {
        raw_defending_ability
    };
    let target_ability_ignored = state.active_ignores_target_ability(
        side_ref,
        &choice.move_id,
        defending_ability,
        choice.category == MoveCategory::Status,
    );
    let (attacking_side, defending_side) = state.get_both_sides(side_ref);
    let defending_index = defending_side.active_index;
    let active_ability = if active_ability_is_suppressed {
        Abilities::NONE
    } else {
        attacking_side.get_active_immutable().ability
    };
    let defender_has_substitute = defending_side
        .volatile_statuses
        .contains(&PokemonVolatileStatus::SUBSTITUTE);
    let active_pkmn = attacking_side.get_active();
    let defending_pkmn = defending_side.get_active();

    let defending_ability = if target_ability_ignored {
        Abilities::NONE
    } else {
        defending_ability
    };
    match defending_ability {
        Abilities::ICEFACE => {
            let hit_substitute = defender_has_substitute
                && !choice.flags.sound
                && active_ability != Abilities::INFILTRATOR;
            if defending_pkmn.id == PokemonName::EISCUE
                && choice.category == MoveCategory::Physical
                && !hit_substitute
            {
                choice.damage_blocked = true;
            }
        }
        // Technically incorrect
        // A move missing should not trigger this formechange
        #[cfg(not(any(feature = "gen8", feature = "gen9")))]
        Abilities::DISGUISE
            if (choice.category == MoveCategory::Physical
                || choice.category == MoveCategory::Special)
                && (defending_pkmn.id == PokemonName::MIMIKYU
                    || defending_pkmn.id == PokemonName::MIMIKYUTOTEM) =>
        {
            choice.base_power = 0.0;
            instructions
                .instruction_list
                .push(Instruction::FormeChange(FormeChangeInstruction {
                    side_ref: side_ref.get_other_side(),
                    name_change: PokemonName::MIMIKYUBUSTED as i16 - defending_pkmn.id as i16,
                }));
            defending_pkmn.id = PokemonName::MIMIKYUBUSTED;
        }
        #[cfg(any(feature = "gen8", feature = "gen9"))]
        Abilities::DISGUISE
            if (choice.category == MoveCategory::Physical
                || choice.category == MoveCategory::Special)
                && (defending_pkmn.id == PokemonName::MIMIKYU
                    || defending_pkmn.id == PokemonName::MIMIKYUTOTEM) =>
        {
            choice.base_power = 0.0;
            instructions
                .instruction_list
                .push(Instruction::FormeChange(FormeChangeInstruction {
                    side_ref: side_ref.get_other_side(),
                    name_change: PokemonName::MIMIKYUBUSTED as i16 - defending_pkmn.id as i16,
                }));
            defending_pkmn.id = PokemonName::MIMIKYUBUSTED;
            let dmg = cmp::min(defending_pkmn.hp, defending_pkmn.maxhp / 8);
            instructions
                .instruction_list
                .push(Instruction::DamageWithFaintContext(
                    DamageWithFaintContextInstruction {
                        side_ref: side_ref.get_other_side(),
                        damage_amount: dmg,
                        faint_context: FaintContext::ability_effect(
                            side_ref.get_other_side(),
                            defending_index,
                            Abilities::DISGUISE,
                        ),
                    },
                ));
            defending_pkmn.hp -= dmg;
        }
        _ => {}
    }
    match active_ability {
        Abilities::GULPMISSILE => {
            if active_pkmn.id == PokemonName::CRAMORANT
                && (choice.move_id == Choices::SURF || choice.move_id == Choices::DIVE)
            {
                let new_forme = if active_pkmn.hp > active_pkmn.maxhp / 2 {
                    PokemonName::CRAMORANTGULPING
                } else {
                    PokemonName::CRAMORANTGORGING
                };
                instructions.instruction_list.push(Instruction::FormeChange(
                    FormeChangeInstruction {
                        side_ref: *side_ref,
                        name_change: new_forme as i16 - active_pkmn.id as i16,
                    },
                ));
                active_pkmn.id = new_forme;
            }
        }
        #[cfg(feature = "gen9")]
        Abilities::PROTEAN | Abilities::LIBERO => {
            if !attacking_side
                .volatile_statuses
                .contains(&PokemonVolatileStatus::TYPECHANGE)
            {
                let active_pkmn = attacking_side.get_active();
                if !active_pkmn.has_type(&choice.move_type) && !active_pkmn.terastallized {
                    instructions
                        .instruction_list
                        .push(Instruction::ChangeType(ChangeType {
                            side_ref: *side_ref,
                            new_types: (choice.move_type, PokemonType::TYPELESS),
                            old_types: active_pkmn.types,
                        }));
                    active_pkmn.types = (choice.move_type, PokemonType::TYPELESS);

                    instructions
                        .instruction_list
                        .push(Instruction::ApplyVolatileStatus(
                            ApplyVolatileStatusInstruction {
                                side_ref: *side_ref,
                                volatile_status: PokemonVolatileStatus::TYPECHANGE,
                            },
                        ));
                    attacking_side
                        .volatile_statuses
                        .insert(PokemonVolatileStatus::TYPECHANGE);
                }
            }
        }
        #[cfg(any(feature = "gen6", feature = "gen7", feature = "gen8"))]
        Abilities::PROTEAN | Abilities::LIBERO => {
            if !active_pkmn.has_type(&choice.move_type) && !active_pkmn.terastallized {
                instructions
                    .instruction_list
                    .push(Instruction::ChangeType(ChangeType {
                        side_ref: *side_ref,
                        new_types: (choice.move_type, PokemonType::TYPELESS),
                        old_types: active_pkmn.types,
                    }));
                active_pkmn.types = (choice.move_type, PokemonType::TYPELESS);
                if !attacking_side
                    .volatile_statuses
                    .contains(&PokemonVolatileStatus::TYPECHANGE)
                {
                    instructions
                        .instruction_list
                        .push(Instruction::ApplyVolatileStatus(
                            ApplyVolatileStatusInstruction {
                                side_ref: *side_ref,
                                volatile_status: PokemonVolatileStatus::TYPECHANGE,
                            },
                        ));
                    attacking_side
                        .volatile_statuses
                        .insert(PokemonVolatileStatus::TYPECHANGE);
                }
            }
        }
        Abilities::GORILLATACTICS => {
            let ins = get_choice_move_disable_instructions(active_pkmn, side_ref, &choice.move_id);
            for i in ins {
                state.apply_one_instruction(&i);
                instructions.instruction_list.push(i);
            }
        }
        _ => {}
    }
}

pub fn ability_after_damage_hit(
    state: &mut State,
    choice: &mut Choice,
    side_ref: &SideReference,
    damage_dealt: i16,
    instructions: &mut StateInstructions,
) {
    let defending_side_ref = side_ref.get_other_side();
    let active_context = state.active_context(side_ref);
    let defending_context = state.active_context(&defending_side_ref);
    let active_item_is_active = active_context.item_is_active;
    let defending_item_is_active = defending_context.item_is_active;
    let (attacking_side, defending_side) = state.get_both_sides(side_ref);
    let active_pkmn = attacking_side.get_active();
    let active_ability = active_context.ability;
    let defending_item_can_be_removed = defending_context.ability != Abilities::STICKYHOLD
        && !defending_side.get_active_immutable().item_is_permanent();
    match active_ability {
        Abilities::BATTLEBOND => {
            if damage_dealt > 0 && defending_side.get_active_immutable().hp == 0 {
                apply_boost_instruction_with_effective(
                    attacking_side,
                    &PokemonBoostableStat::Attack,
                    &1,
                    side_ref,
                    side_ref,
                    active_ability,
                    active_item_is_active,
                    instructions,
                );
                apply_boost_instruction_with_effective(
                    attacking_side,
                    &PokemonBoostableStat::SpecialAttack,
                    &1,
                    side_ref,
                    side_ref,
                    active_ability,
                    active_item_is_active,
                    instructions,
                );
                apply_boost_instruction_with_effective(
                    attacking_side,
                    &PokemonBoostableStat::Speed,
                    &1,
                    side_ref,
                    side_ref,
                    active_ability,
                    active_item_is_active,
                    instructions,
                );
            }
        }
        Abilities::MAGICIAN | Abilities::PICKPOCKET => {
            let defending_pkmn = defending_side.get_active();
            if damage_dealt > 0 && defending_item_can_be_removed && active_pkmn.item == Items::NONE
            {
                instructions.instruction_list.push(Instruction::ChangeItem(
                    ChangeItemInstruction {
                        side_ref: *side_ref,
                        current_item: active_pkmn.item,
                        new_item: defending_pkmn.item,
                    },
                ));
                active_pkmn.item = defending_pkmn.item;
                instructions.instruction_list.push(Instruction::ChangeItem(
                    ChangeItemInstruction {
                        side_ref: side_ref.get_other_side(),
                        current_item: defending_pkmn.item,
                        new_item: Items::NONE,
                    },
                ));
                defending_pkmn.item = Items::NONE;
            }
        }
        Abilities::MOXIE | Abilities::CHILLINGNEIGH | Abilities::ASONEGLASTRIER => {
            if damage_dealt > 0 && defending_side.get_active_immutable().hp == 0 {
                apply_boost_instruction_with_effective(
                    attacking_side,
                    &PokemonBoostableStat::Attack,
                    &1,
                    side_ref,
                    side_ref,
                    active_ability,
                    active_item_is_active,
                    instructions,
                );
            }
        }
        Abilities::GRIMNEIGH | Abilities::ASONESPECTRIER => {
            if damage_dealt > 0 && defending_side.get_active_immutable().hp == 0 {
                apply_boost_instruction_with_effective(
                    attacking_side,
                    &PokemonBoostableStat::SpecialAttack,
                    &1,
                    side_ref,
                    side_ref,
                    active_ability,
                    active_item_is_active,
                    instructions,
                );
            }
        }
        Abilities::BEASTBOOST => {
            if damage_dealt > 0 && defending_side.get_active_immutable().hp == 0 {
                let highest_stat = &attacking_side.calculate_highest_stat();
                apply_boost_instruction_with_effective(
                    attacking_side,
                    highest_stat,
                    &1,
                    side_ref,
                    side_ref,
                    active_ability,
                    active_item_is_active,
                    instructions,
                );
            }
        }
        _ => {}
    }
    let active_change_context = state.active_context(side_ref);
    let defending_change_context = state.active_context(&defending_side_ref);
    let (attacking_side, defending_side) = state.get_both_sides(side_ref);
    let defending_index = defending_side.active_index;
    let attacking_pkmn = attacking_side.get_active();
    let defending_pkmn = defending_side.get_active();
    let attacking_ability_can_be_changed =
        attacking_pkmn.hp != 0 && !active_change_context.has_effective_item(Items::ABILITYSHIELD);
    let defending_ability_can_be_changed = defending_pkmn.hp != 0
        && !defending_change_context.has_effective_item(Items::ABILITYSHIELD);
    let defending_ability = defending_context.ability;
    match defending_ability {
        Abilities::MUMMY => {
            if choice.flags.contact
                && attacking_ability_can_be_changed
                && attacking_pkmn.ability != Abilities::MUMMY
                && !ability_cannot_be_replaced_by_contact(attacking_pkmn.ability)
            {
                instructions
                    .instruction_list
                    .push(Instruction::ChangeAbility(ChangeAbilityInstruction {
                        side_ref: *side_ref,
                        ability_change: Abilities::MUMMY as i16 - attacking_pkmn.ability as i16,
                    }));
                attacking_pkmn.ability = Abilities::MUMMY;
            }
        }
        Abilities::LINGERINGAROMA => {
            if choice.flags.contact
                && attacking_ability_can_be_changed
                && attacking_pkmn.ability != Abilities::LINGERINGAROMA
                && !ability_cannot_be_replaced_by_contact(attacking_pkmn.ability)
            {
                instructions
                    .instruction_list
                    .push(Instruction::ChangeAbility(ChangeAbilityInstruction {
                        side_ref: *side_ref,
                        ability_change: Abilities::LINGERINGAROMA as i16
                            - attacking_pkmn.ability as i16,
                    }));
                attacking_pkmn.ability = Abilities::LINGERINGAROMA;
            }
        }
        Abilities::WANDERINGSPIRIT => {
            if choice.flags.contact
                && attacking_ability_can_be_changed
                && defending_ability_can_be_changed
                && !ability_fails_skill_swap(attacking_pkmn.ability)
                && !ability_fails_skill_swap(defending_pkmn.ability)
            {
                let attacking_ability = attacking_pkmn.ability;
                let defending_ability = defending_pkmn.ability;
                instructions
                    .instruction_list
                    .push(Instruction::ChangeAbility(ChangeAbilityInstruction {
                        side_ref: *side_ref,
                        ability_change: defending_ability as i16 - attacking_ability as i16,
                    }));
                instructions
                    .instruction_list
                    .push(Instruction::ChangeAbility(ChangeAbilityInstruction {
                        side_ref: side_ref.get_other_side(),
                        ability_change: attacking_ability as i16 - defending_ability as i16,
                    }));
                attacking_pkmn.ability = defending_ability;
                defending_pkmn.ability = attacking_ability;
            }
        }
        Abilities::ELECTROMORPHOSIS => {
            if damage_dealt > 0 && defending_pkmn.hp != 0 {
                apply_charge_volatile(defending_side, side_ref.get_other_side(), instructions);
            }
        }
        Abilities::WINDPOWER => {
            if damage_dealt > 0 && defending_pkmn.hp != 0 && choice.flags.wind {
                apply_charge_volatile(defending_side, side_ref.get_other_side(), instructions);
            }
        }
        Abilities::GULPMISSILE => {
            if damage_dealt > 0
                && [PokemonName::CRAMORANTGORGING, PokemonName::CRAMORANTGULPING]
                    .contains(&defending_pkmn.id)
            {
                instructions.instruction_list.push(Instruction::FormeChange(
                    FormeChangeInstruction {
                        side_ref: side_ref.get_other_side(),
                        name_change: PokemonName::CRAMORANT as i16 - defending_pkmn.id as i16,
                    },
                ));

                let damage_dealt = cmp::min(attacking_pkmn.maxhp / 4, attacking_pkmn.hp);
                instructions
                    .instruction_list
                    .push(Instruction::DamageWithFaintContext(
                        DamageWithFaintContextInstruction {
                            side_ref: *side_ref,
                            damage_amount: damage_dealt,
                            faint_context: FaintContext::ability_effect(
                                side_ref.get_other_side(),
                                defending_index,
                                Abilities::GULPMISSILE,
                            ),
                        },
                    ));
                attacking_pkmn.hp -= damage_dealt;

                if defending_pkmn.id == PokemonName::CRAMORANTGULPING {
                    defending_pkmn.id = PokemonName::CRAMORANT;
                    apply_boost_instruction_with_effective(
                        attacking_side,
                        &PokemonBoostableStat::Defense,
                        &-1,
                        &side_ref.get_other_side(),
                        side_ref,
                        active_ability,
                        active_item_is_active,
                        instructions,
                    );
                } else if defending_pkmn.id == PokemonName::CRAMORANTGORGING {
                    defending_pkmn.id = PokemonName::CRAMORANT;
                    choice.add_or_create_secondaries(Secondary {
                        chance: 100.0,
                        target: MoveTarget::User,
                        effect: Effect::Status(PokemonStatus::PARALYZE),
                    })
                }
            }
        }
        Abilities::COLORCHANGE => {
            if damage_dealt > 0
                && defending_pkmn.hp != 0
                && !defending_pkmn.has_type(&choice.move_type)
            {
                let change_type_instruction = Instruction::ChangeType(ChangeType {
                    side_ref: side_ref.get_other_side(),
                    new_types: (choice.move_type, PokemonType::TYPELESS),
                    old_types: defending_pkmn.types,
                });
                defending_pkmn.types = (choice.move_type, PokemonType::TYPELESS);
                instructions.instruction_list.push(change_type_instruction);
            }
        }
        Abilities::STAMINA => {
            if damage_dealt > 0 && defending_pkmn.hp != 0 {
                apply_boost_instruction_with_effective(
                    defending_side,
                    &PokemonBoostableStat::Defense,
                    &1,
                    side_ref,
                    &side_ref.get_other_side(),
                    defending_ability,
                    defending_item_is_active,
                    instructions,
                );
            }
        }
        Abilities::COTTONDOWN => {
            if damage_dealt > 0 {
                apply_boost_instruction_with_effective(
                    attacking_side,
                    &PokemonBoostableStat::Speed,
                    &-1,
                    &side_ref.get_other_side(),
                    side_ref,
                    active_ability,
                    active_item_is_active,
                    instructions,
                );
            }
        }
        Abilities::SANDSPIT => {
            if damage_dealt > 0 && regular_weather_should_be_set(state, Weather::SAND) {
                instructions
                    .instruction_list
                    .push(Instruction::ChangeWeather(ChangeWeather {
                        new_weather: Weather::SAND,
                        new_weather_turns_remaining: WEATHER_ABILITY_TURNS,
                        previous_weather: state.weather.weather_type,
                        previous_weather_turns_remaining: state.weather.turns_remaining,
                    }));
                state.weather.weather_type = Weather::SAND;
                state.weather.turns_remaining = WEATHER_ABILITY_TURNS;
            }
        }
        Abilities::SEEDSOWER => {
            if damage_dealt > 0 && terrain_should_be_set(state, Terrain::GRASSYTERRAIN) {
                instructions
                    .instruction_list
                    .push(Instruction::ChangeTerrain(ChangeTerrain {
                        new_terrain: Terrain::GRASSYTERRAIN,
                        new_terrain_turns_remaining: 5,
                        previous_terrain: state.terrain.terrain_type,
                        previous_terrain_turns_remaining: state.terrain.turns_remaining,
                    }));
                state.terrain.terrain_type = Terrain::GRASSYTERRAIN;
                state.terrain.turns_remaining = 5;
            }
        }
        Abilities::TOXICDEBRIS => {
            // Not complete: Toxic Spikes are not applied if a substitute is hit
            if damage_dealt > 0
                && choice.category == MoveCategory::Physical
                && attacking_side.side_conditions.toxic_spikes < 2
            {
                instructions
                    .instruction_list
                    .push(Instruction::ChangeSideCondition(
                        ChangeSideConditionInstruction {
                            side_ref: *side_ref,
                            side_condition: PokemonSideCondition::ToxicSpikes,
                            amount: 1,
                        },
                    ));
                attacking_side.side_conditions.toxic_spikes += 1;
            }
        }
        Abilities::BERSERK => {
            if damage_dealt > 0
                && defending_pkmn.hp < defending_pkmn.maxhp / 2
                && defending_pkmn.hp + damage_dealt >= defending_pkmn.maxhp / 2
            {
                apply_boost_instruction_with_effective(
                    defending_side,
                    &PokemonBoostableStat::SpecialAttack,
                    &1,
                    &side_ref.get_other_side(),
                    &side_ref.get_other_side(),
                    defending_ability,
                    defending_item_is_active,
                    instructions,
                );
            }
        }
        Abilities::ROUGHSKIN | Abilities::IRONBARBS => {
            if damage_dealt > 0 && choice.flags.contact {
                #[cfg(feature = "gen3")]
                let damage_dealt = cmp::min(attacking_pkmn.maxhp / 16, attacking_pkmn.hp);

                #[cfg(not(feature = "gen3"))]
                let damage_dealt = cmp::min(attacking_pkmn.maxhp / 8, attacking_pkmn.hp);

                instructions
                    .instruction_list
                    .push(Instruction::DamageWithFaintContext(
                        DamageWithFaintContextInstruction {
                            side_ref: *side_ref,
                            damage_amount: damage_dealt,
                            faint_context: FaintContext::ability_effect(
                                side_ref.get_other_side(),
                                defending_index,
                                defending_ability,
                            ),
                        },
                    ));
                attacking_pkmn.hp -= damage_dealt;
            }
        }
        Abilities::SPICYSPRAY => {
            if damage_dealt > 0 {
                apply_spicy_spray_burn(state, side_ref, instructions);
            }
        }
        Abilities::AFTERMATH => {
            if damage_dealt > 0
                && defending_side.get_active_immutable().hp == 0
                && choice.flags.contact
            {
                let damage_dealt = cmp::min(attacking_pkmn.maxhp / 4, attacking_pkmn.hp);
                instructions
                    .instruction_list
                    .push(Instruction::DamageWithFaintContext(
                        DamageWithFaintContextInstruction {
                            side_ref: *side_ref,
                            damage_amount: damage_dealt,
                            faint_context: FaintContext::ability_effect(
                                side_ref.get_other_side(),
                                defending_index,
                                Abilities::AFTERMATH,
                            ),
                        },
                    ));
                attacking_pkmn.hp -= damage_dealt;
            }
        }
        Abilities::INNARDSOUT => {
            if damage_dealt > 0 && defending_side.get_active_immutable().hp == 0 {
                let damage_dealt = cmp::min(damage_dealt, attacking_pkmn.hp);
                instructions
                    .instruction_list
                    .push(Instruction::DamageWithFaintContext(
                        DamageWithFaintContextInstruction {
                            side_ref: *side_ref,
                            damage_amount: damage_dealt,
                            faint_context: FaintContext::ability_effect(
                                side_ref.get_other_side(),
                                defending_index,
                                Abilities::INNARDSOUT,
                            ),
                        },
                    ));
                attacking_pkmn.hp -= damage_dealt;
            }
        }
        Abilities::PERISHBODY => {
            if damage_dealt > 0 && choice.flags.contact {
                for side_ref in [SideReference::SideOne, SideReference::SideTwo] {
                    let soundproof_active =
                        state.active_ability(&side_ref) == Abilities::SOUNDPROOF;
                    let side = state.get_side(&side_ref);
                    let pkmn = side.get_active();
                    if pkmn.hp != 0
                        && !soundproof_active
                        && !(side
                            .volatile_statuses
                            .contains(&PokemonVolatileStatus::PERISH4)
                            || side
                                .volatile_statuses
                                .contains(&PokemonVolatileStatus::PERISH3)
                            || side
                                .volatile_statuses
                                .contains(&PokemonVolatileStatus::PERISH2)
                            || side
                                .volatile_statuses
                                .contains(&PokemonVolatileStatus::PERISH1))
                    {
                        instructions
                            .instruction_list
                            .push(Instruction::ApplyVolatileStatus(
                                ApplyVolatileStatusInstruction {
                                    side_ref: side_ref,
                                    volatile_status: PokemonVolatileStatus::PERISH4,
                                },
                            ));
                        side.volatile_statuses
                            .insert(PokemonVolatileStatus::PERISH4);
                    }
                }
            }
        }
        _ => {}
    }
}

pub fn ability_after_substitute_hit(
    attacking_side: &mut Side,
    defending_side: &Side,
    defending_ability: Abilities,
    choice: &Choice,
    side_ref: &SideReference,
    damage_dealt: i16,
    instructions: &mut StateInstructions,
) {
    if defending_ability == Abilities::TOXICDEBRIS
        && damage_dealt > 0
        && choice.category == MoveCategory::Physical
        && attacking_side.side_conditions.toxic_spikes < 2
    {
        instructions
            .instruction_list
            .push(Instruction::ChangeSideCondition(
                ChangeSideConditionInstruction {
                    side_ref: *side_ref,
                    side_condition: PokemonSideCondition::ToxicSpikes,
                    amount: 1,
                },
            ));
        attacking_side.side_conditions.toxic_spikes += 1;
    }
}

pub fn ability_on_damage_blocked(
    defending_side: &mut Side,
    choice: &Choice,
    defending_side_ref: &SideReference,
    instructions: &mut StateInstructions,
) -> bool {
    let defending_pkmn = defending_side.get_active();

    if choice.damage_blocked
        && defending_pkmn.ability == Abilities::ICEFACE
        && defending_pkmn.id == PokemonName::EISCUE
    {
        instructions
            .instruction_list
            .push(Instruction::FormeChange(FormeChangeInstruction {
                side_ref: *defending_side_ref,
                name_change: PokemonName::EISCUENOICE as i16 - defending_pkmn.id as i16,
            }));
        defending_pkmn.id = PokemonName::EISCUENOICE;
        defending_pkmn.recalculate_stats(defending_side_ref, instructions);
        return true;
    }
    false
}

fn apply_spicy_spray_burn(
    state: &mut State,
    side_ref: &SideReference,
    instructions: &mut StateInstructions,
) {
    if immune_to_status(state, &MoveTarget::User, side_ref, &PokemonStatus::BURN) {
        return;
    }

    let berries_blocked = opponent_blocks_berries(state, side_ref);
    let cheek_pouch_active = state.active_ability_is_active(side_ref, Abilities::CHEEKPOUCH);
    let item_is_active = state.active_item_is_active(side_ref);
    let attacking_side = state.get_side(side_ref);
    let active_index = attacking_side.active_index;
    let attacking_pkmn = attacking_side.get_active();

    if attacking_pkmn.item == Items::LUMBERRY && item_is_active && !berries_blocked {
        consume_berry(
            side_ref,
            attacking_pkmn,
            Items::LUMBERRY,
            cheek_pouch_active,
            instructions,
        );
    } else {
        instructions
            .instruction_list
            .push(Instruction::ChangeStatus(ChangeStatusInstruction {
                side_ref: *side_ref,
                pokemon_index: active_index,
                old_status: attacking_pkmn.status,
                new_status: PokemonStatus::BURN,
            }));
        attacking_pkmn.status = PokemonStatus::BURN;
    }
}

pub fn ability_on_switch_out(
    state: &mut State,
    side_ref: &SideReference,
    instructions: &mut StateInstructions,
) {
    let active_ability_is_suppressed = state.active_ability_is_suppressed(side_ref);
    let other_side_ref = side_ref.get_other_side();
    let other_active = state
        .get_side_immutable(&other_side_ref)
        .get_active_immutable();
    let other_active_hp = other_active.hp;
    let other_active_ability = state.active_ability(&other_side_ref);
    let (attacking_side, defending_side) = state.get_both_sides(side_ref);
    let active_pkmn = attacking_side.get_active();
    let active_ability = if active_ability_is_suppressed {
        Abilities::NONE
    } else {
        active_pkmn.ability
    };
    match active_ability {
        Abilities::GULPMISSILE if active_pkmn.base_ability == Abilities::GULPMISSILE => {
            if active_pkmn.id != PokemonName::CRAMORANT {
                instructions.instruction_list.push(Instruction::FormeChange(
                    FormeChangeInstruction {
                        side_ref: *side_ref,
                        name_change: PokemonName::CRAMORANT as i16 - active_pkmn.id as i16,
                    },
                ));
                active_pkmn.id = PokemonName::CRAMORANT;
            }
        }
        Abilities::ZEROTOHERO => {
            if active_pkmn.id == PokemonName::PALAFIN {
                instructions.instruction_list.push(Instruction::FormeChange(
                    FormeChangeInstruction {
                        side_ref: *side_ref,
                        name_change: PokemonName::PALAFINHERO as i16 - active_pkmn.id as i16,
                    },
                ));
                active_pkmn.id = PokemonName::PALAFINHERO;
                active_pkmn.recalculate_stats(side_ref, instructions);
            }
        }
        Abilities::HUNGERSWITCH => {
            if active_pkmn.id == PokemonName::MORPEKOHANGRY && !active_pkmn.terastallized {
                instructions.instruction_list.push(Instruction::FormeChange(
                    FormeChangeInstruction {
                        side_ref: *side_ref,
                        name_change: PokemonName::MORPEKO as i16 - active_pkmn.id as i16,
                    },
                ));
                active_pkmn.id = PokemonName::MORPEKO;
            }
        }
        Abilities::NATURALCURE => {
            if active_pkmn.status != PokemonStatus::NONE {
                let status = active_pkmn.status.clone();
                active_pkmn.status = PokemonStatus::NONE;
                instructions
                    .instruction_list
                    .push(Instruction::ChangeStatus(ChangeStatusInstruction {
                        side_ref: *side_ref,
                        pokemon_index: attacking_side.active_index,
                        old_status: status,
                        new_status: PokemonStatus::NONE,
                    }));
            }
        }
        Abilities::REGENERATOR => {
            let hp_recovered = cmp::min(active_pkmn.maxhp / 3, active_pkmn.maxhp - active_pkmn.hp);

            if hp_recovered > 0 && active_pkmn.hp > 0 {
                instructions
                    .instruction_list
                    .push(Instruction::Heal(HealInstruction {
                        side_ref: *side_ref,
                        heal_amount: hp_recovered,
                    }));
                active_pkmn.hp += hp_recovered;
            }
        }
        Abilities::PRIMORDIALSEA => {
            if state.weather.weather_type == Weather::HEAVYRAIN
                && (other_active_hp == 0 || other_active_ability != Abilities::PRIMORDIALSEA)
            {
                instructions
                    .instruction_list
                    .push(Instruction::ChangeWeather(ChangeWeather {
                        new_weather: Weather::NONE,
                        new_weather_turns_remaining: -1,
                        previous_weather: state.weather.weather_type,
                        previous_weather_turns_remaining: state.weather.turns_remaining,
                    }));
                state.weather.weather_type = Weather::NONE;
                state.weather.turns_remaining = -1;
            }
        }
        Abilities::DESOLATELAND => {
            if state.weather.weather_type == Weather::HARSHSUN
                && (other_active_hp == 0 || other_active_ability != Abilities::DESOLATELAND)
            {
                instructions
                    .instruction_list
                    .push(Instruction::ChangeWeather(ChangeWeather {
                        new_weather: Weather::NONE,
                        new_weather_turns_remaining: -1,
                        previous_weather: state.weather.weather_type,
                        previous_weather_turns_remaining: state.weather.turns_remaining,
                    }));
                state.weather.weather_type = Weather::NONE;
                state.weather.turns_remaining = -1;
            }
        }
        _ => {}
    }

    // revert ability on switch-out to base_ability if they are not the same
    let active_pkmn = state.get_side(side_ref).get_active();
    if active_pkmn.ability != active_pkmn.base_ability {
        instructions
            .instruction_list
            .push(Instruction::ChangeAbility(ChangeAbilityInstruction {
                side_ref: *side_ref,
                ability_change: active_pkmn.base_ability as i16 - active_pkmn.ability as i16,
            }));
        active_pkmn.ability = active_pkmn.base_ability;
    }
}

pub fn ability_end_of_turn(
    state: &mut State,
    side_ref: &SideReference,
    instructions: &mut StateInstructions,
) {
    let sun_is_active =
        state.weather_is_active(&Weather::HARSHSUN) || state.weather_is_active(&Weather::SUN);
    let rain_is_active =
        state.weather_is_active(&Weather::HEAVYRAIN) || state.weather_is_active(&Weather::RAIN);
    let hail_or_snow_is_active =
        state.weather_is_active(&Weather::HAIL) || state.weather_is_active(&Weather::SNOW);
    let active_ability_is_suppressed = state.active_ability_is_suppressed(side_ref);
    let (attacking_side, defending_side) = state.get_both_sides(side_ref);
    let attacking_index = attacking_side.active_index;
    let active_pkmn = attacking_side.get_active();
    let active_ability = if active_ability_is_suppressed {
        Abilities::NONE
    } else {
        active_pkmn.ability
    };
    match active_ability {
        Abilities::HUNGERSWITCH => {
            if active_pkmn.id == PokemonName::MORPEKO && !active_pkmn.terastallized {
                instructions.instruction_list.push(Instruction::FormeChange(
                    FormeChangeInstruction {
                        side_ref: *side_ref,
                        name_change: PokemonName::MORPEKOHANGRY as i16 - active_pkmn.id as i16,
                    },
                ));
                active_pkmn.id = PokemonName::MORPEKOHANGRY;
            } else if active_pkmn.id == PokemonName::MORPEKOHANGRY && !active_pkmn.terastallized {
                instructions.instruction_list.push(Instruction::FormeChange(
                    FormeChangeInstruction {
                        side_ref: *side_ref,
                        name_change: PokemonName::MORPEKO as i16 - active_pkmn.id as i16,
                    },
                ));
                active_pkmn.id = PokemonName::MORPEKO;
            }
        }
        Abilities::SHIELDSDOWN => {
            if active_pkmn.hp <= active_pkmn.maxhp / 2
                && active_pkmn.id == PokemonName::MINIORMETEOR
            {
                instructions.instruction_list.push(Instruction::FormeChange(
                    FormeChangeInstruction {
                        side_ref: *side_ref,
                        name_change: PokemonName::MINIOR as i16 - active_pkmn.id as i16,
                    },
                ));
                active_pkmn.id = PokemonName::MINIOR;
                active_pkmn.recalculate_stats(side_ref, instructions);
            }
            if active_pkmn.hp > active_pkmn.maxhp / 2 && active_pkmn.id != PokemonName::MINIORMETEOR
            {
                instructions.instruction_list.push(Instruction::FormeChange(
                    FormeChangeInstruction {
                        side_ref: *side_ref,
                        name_change: PokemonName::MINIORMETEOR as i16 - active_pkmn.id as i16,
                    },
                ));
                active_pkmn.id = PokemonName::MINIORMETEOR;
                active_pkmn.recalculate_stats(side_ref, instructions);
            }
        }
        Abilities::SCHOOLING => {
            if active_pkmn.hp <= active_pkmn.maxhp / 4
                && active_pkmn.id == PokemonName::WISHIWASHISCHOOL
            {
                instructions.instruction_list.push(Instruction::FormeChange(
                    FormeChangeInstruction {
                        side_ref: *side_ref,
                        name_change: PokemonName::WISHIWASHI as i16 - active_pkmn.id as i16,
                    },
                ));
                active_pkmn.id = PokemonName::WISHIWASHI;
                active_pkmn.recalculate_stats(side_ref, instructions);
            }
            if active_pkmn.hp > active_pkmn.maxhp / 4 && active_pkmn.id == PokemonName::WISHIWASHI {
                instructions.instruction_list.push(Instruction::FormeChange(
                    FormeChangeInstruction {
                        side_ref: *side_ref,
                        name_change: PokemonName::WISHIWASHISCHOOL as i16 - active_pkmn.id as i16,
                    },
                ));
                active_pkmn.id = PokemonName::WISHIWASHISCHOOL;
                active_pkmn.recalculate_stats(side_ref, instructions);
            }
        }
        Abilities::BADDREAMS => {
            let defender = defending_side.get_active();
            if defender.status == PokemonStatus::SLEEP {
                let damage_dealt = cmp::min(defender.maxhp / 8, defender.hp);
                instructions
                    .instruction_list
                    .push(Instruction::DamageWithFaintContext(
                        DamageWithFaintContextInstruction {
                            side_ref: side_ref.get_other_side(),
                            damage_amount: damage_dealt,
                            faint_context: FaintContext::ability_effect(
                                *side_ref,
                                attacking_index,
                                Abilities::BADDREAMS,
                            ),
                        },
                    ));
                defender.hp -= damage_dealt;
            }
        }
        Abilities::SOLARPOWER => {
            if sun_is_active {
                let damage_dealt =
                    cmp::min(active_pkmn.maxhp / 8, active_pkmn.maxhp - active_pkmn.hp);
                if damage_dealt > 0 {
                    instructions
                        .instruction_list
                        .push(Instruction::DamageWithFaintContext(
                            DamageWithFaintContextInstruction {
                                side_ref: *side_ref,
                                damage_amount: damage_dealt,
                                faint_context: FaintContext::ability_effect(
                                    *side_ref,
                                    attacking_index,
                                    Abilities::SOLARPOWER,
                                ),
                            },
                        ));
                    active_pkmn.hp -= damage_dealt;
                }
            }
        }
        Abilities::ICEBODY => {
            if hail_or_snow_is_active {
                let active_pkmn = state.get_side(side_ref).get_active();
                let health_recovered =
                    cmp::min(active_pkmn.maxhp / 16, active_pkmn.maxhp - active_pkmn.hp);
                if health_recovered > 0 {
                    instructions
                        .instruction_list
                        .push(Instruction::Heal(HealInstruction {
                            side_ref: *side_ref,
                            heal_amount: health_recovered,
                        }));
                    active_pkmn.hp += health_recovered;
                }
            }
        }
        Abilities::POISONHEAL => {
            if active_pkmn.hp < active_pkmn.maxhp
                && (active_pkmn.status == PokemonStatus::POISON
                    || active_pkmn.status == PokemonStatus::TOXIC)
            {
                let heal_amount =
                    cmp::min(active_pkmn.maxhp / 8, active_pkmn.maxhp - active_pkmn.hp);
                let ins = Instruction::Heal(HealInstruction {
                    side_ref: side_ref.clone(),
                    heal_amount: heal_amount,
                });
                active_pkmn.hp += heal_amount;
                instructions.instruction_list.push(ins);
            }
        }
        Abilities::SPEEDBOOST => {
            if attacking_side.speed_boost < 6 {
                let ins = Instruction::Boost(BoostInstruction {
                    side_ref: side_ref.clone(),
                    stat: PokemonBoostableStat::Speed,
                    amount: 1,
                });
                attacking_side.speed_boost += 1;
                instructions.instruction_list.push(ins);
            }
        }
        Abilities::RAINDISH => {
            if rain_is_active {
                let active_pkmn = state.get_side(side_ref).get_active();
                let health_recovered =
                    cmp::min(active_pkmn.maxhp / 16, active_pkmn.maxhp - active_pkmn.hp);
                if health_recovered > 0 {
                    instructions
                        .instruction_list
                        .push(Instruction::Heal(HealInstruction {
                            side_ref: *side_ref,
                            heal_amount: health_recovered,
                        }));
                    active_pkmn.hp += health_recovered;
                }
            }
        }
        Abilities::DRYSKIN => {
            if rain_is_active {
                if active_pkmn.hp < active_pkmn.maxhp {
                    let heal_amount =
                        cmp::min(active_pkmn.maxhp / 8, active_pkmn.maxhp - active_pkmn.hp);
                    let ins = Instruction::Heal(HealInstruction {
                        side_ref: side_ref.clone(),
                        heal_amount: heal_amount,
                    });
                    active_pkmn.hp += heal_amount;
                    instructions.instruction_list.push(ins);
                }
            } else if sun_is_active {
                let damage_dealt = cmp::min(active_pkmn.maxhp / 8, active_pkmn.hp);
                if damage_dealt > 0 {
                    instructions
                        .instruction_list
                        .push(Instruction::DamageWithFaintContext(
                            DamageWithFaintContextInstruction {
                                side_ref: *side_ref,
                                damage_amount: damage_dealt,
                                faint_context: FaintContext::ability_effect(
                                    *side_ref,
                                    attacking_index,
                                    Abilities::DRYSKIN,
                                ),
                            },
                        ));
                    active_pkmn.hp -= damage_dealt;
                }
            }
        }
        Abilities::HYDRATION => {
            if active_pkmn.status != PokemonStatus::NONE && rain_is_active {
                let attacking_side = state.get_side(side_ref);
                let active_index = attacking_side.active_index;
                let active_pkmn = attacking_side.get_active();

                add_remove_status_instructions(
                    instructions,
                    active_index,
                    *side_ref,
                    attacking_side,
                );
            }
        }
        // Shed skin only has a 1/3 chance of activating at the end of the turn
        // but I'm not going to branch on that here
        Abilities::SHEDSKIN => {
            if active_pkmn.status != PokemonStatus::NONE {
                let attacking_side = state.get_side(side_ref);
                let active_index = attacking_side.active_index;
                let active_pkmn = attacking_side.get_active();

                add_remove_status_instructions(
                    instructions,
                    active_index,
                    *side_ref,
                    attacking_side,
                );
            }
        }
        _ => {}
    }
}

pub fn ability_on_start(
    state: &mut State,
    side_ref: &SideReference,
    instructions: &mut StateInstructions,
) {
    let active_ability_is_active = !state.active_ability_is_suppressed(side_ref);
    let defender_side_ref = side_ref.get_other_side();
    let active_item_is_active = state.active_item_is_active(side_ref);
    let active_ability_can_be_changed = state.active_ability_can_be_changed(side_ref);
    let defender_ability = state.active_ability(&defender_side_ref);
    let defender_item_is_active = state.active_item_is_active(&defender_side_ref);
    let (attacking_side, defending_side) = state.get_both_sides(side_ref);
    let active_index = attacking_side.active_index;
    let active_pkmn = attacking_side.get_active();
    let defending_pkmn = defending_side.get_active_immutable();

    // trace copying an ability needs to happen before the ability check to activate on switch-in
    // e.g. tracing intimidate will activate intimidate
    if active_ability_is_active
        && active_pkmn.ability == Abilities::TRACE
        && active_pkmn.ability != defending_pkmn.ability
        && active_ability_can_be_changed
        && !ability_fails_trace(defending_pkmn.ability)
    {
        instructions
            .instruction_list
            .push(Instruction::ChangeAbility(ChangeAbilityInstruction {
                side_ref: *side_ref,
                ability_change: defending_pkmn.ability as i16 - active_pkmn.ability as i16,
            }));
        active_pkmn.ability = defending_pkmn.ability;
    }

    let active_ability = if active_ability_is_active {
        active_pkmn.ability
    } else {
        Abilities::NONE
    };
    match active_ability {
        Abilities::ICEFACE => {
            if active_pkmn.id == PokemonName::EISCUENOICE
                && (state.weather_is_active(&Weather::HAIL)
                    || state.weather_is_active(&Weather::SNOW))
            {
                let active_pkmn = state.get_side(side_ref).get_active();
                instructions.instruction_list.push(Instruction::FormeChange(
                    FormeChangeInstruction {
                        side_ref: *side_ref,
                        name_change: PokemonName::EISCUE as i16 - active_pkmn.id as i16,
                    },
                ));
                active_pkmn.id = PokemonName::EISCUE;
                active_pkmn.recalculate_stats(side_ref, instructions);
            }
        }
        Abilities::PROTOSYNTHESIS => {
            let sun_is_active = state.weather_is_active(&Weather::SUN);
            let item_is_active = state.active_item_is_active(side_ref);
            let attacking_side = state.get_side(side_ref);
            let volatile = protosynthesis_volatile_from_side(&attacking_side);
            protosynthesus_or_quarkdrive_on_switch_in(
                sun_is_active,
                item_is_active,
                volatile,
                PokemonVolatileStatus::PROTOSYNTHESISBOOSTER,
                instructions,
                attacking_side,
                side_ref,
            );
        }
        Abilities::QUARKDRIVE => {
            let electric_terrain_is_active = state.terrain_is_active(&Terrain::ELECTRICTERRAIN);
            let item_is_active = state.active_item_is_active(side_ref);
            let attacking_side = state.get_side(side_ref);
            let volatile = quarkdrive_volatile_from_side(&attacking_side);
            protosynthesus_or_quarkdrive_on_switch_in(
                electric_terrain_is_active,
                item_is_active,
                volatile,
                PokemonVolatileStatus::QUARKDRIVEBOOSTER,
                instructions,
                attacking_side,
                side_ref,
            );
        }
        Abilities::EMBODYASPECTTEAL => {
            apply_boost_instruction_with_effective(
                attacking_side,
                &PokemonBoostableStat::Speed,
                &1,
                side_ref,
                side_ref,
                active_ability,
                active_item_is_active,
                instructions,
            );
        }
        Abilities::EMBODYASPECTWELLSPRING => {
            apply_boost_instruction_with_effective(
                attacking_side,
                &PokemonBoostableStat::SpecialDefense,
                &1,
                side_ref,
                side_ref,
                active_ability,
                active_item_is_active,
                instructions,
            );
        }
        Abilities::EMBODYASPECTCORNERSTONE => {
            apply_boost_instruction_with_effective(
                attacking_side,
                &PokemonBoostableStat::Defense,
                &1,
                side_ref,
                side_ref,
                active_ability,
                active_item_is_active,
                instructions,
            );
        }
        Abilities::EMBODYASPECTHEARTHFLAME => {
            apply_boost_instruction_with_effective(
                attacking_side,
                &PokemonBoostableStat::Attack,
                &1,
                side_ref,
                side_ref,
                active_ability,
                active_item_is_active,
                instructions,
            );
        }
        Abilities::INTREPIDSWORD => {
            #[cfg(feature = "gen9")]
            if !active_pkmn.sword_boost_used {
                instructions
                    .instruction_list
                    .push(Instruction::ToggleSwordBoostUsed(
                        ToggleAbilityOnStartFlagInstruction {
                            side_ref: *side_ref,
                            pokemon_index: active_index,
                        },
                    ));
                active_pkmn.sword_boost_used = true;
                apply_boost_instruction_with_effective(
                    attacking_side,
                    &PokemonBoostableStat::Attack,
                    &1,
                    side_ref,
                    side_ref,
                    active_ability,
                    active_item_is_active,
                    instructions,
                );
            }
            #[cfg(not(feature = "gen9"))]
            {
                apply_boost_instruction_with_effective(
                    attacking_side,
                    &PokemonBoostableStat::Attack,
                    &1,
                    side_ref,
                    side_ref,
                    active_ability,
                    active_item_is_active,
                    instructions,
                );
            }
        }
        Abilities::SLOWSTART => {
            instructions
                .instruction_list
                .push(Instruction::ApplyVolatileStatus(
                    ApplyVolatileStatusInstruction {
                        side_ref: *side_ref,
                        volatile_status: PokemonVolatileStatus::SLOWSTART,
                    },
                ));
            instructions
                .instruction_list
                .push(Instruction::ChangeVolatileStatusDuration(
                    ChangeVolatileStatusDurationInstruction {
                        side_ref: *side_ref,
                        volatile_status: PokemonVolatileStatus::SLOWSTART,
                        amount: 6 - attacking_side.volatile_status_durations.slowstart,
                    },
                ));
            attacking_side
                .volatile_statuses
                .insert(PokemonVolatileStatus::SLOWSTART);
            attacking_side.volatile_status_durations.slowstart = 6;
        }
        Abilities::DROUGHT | Abilities::ORICHALCUMPULSE => {
            if regular_weather_should_be_set(state, Weather::SUN) {
                instructions
                    .instruction_list
                    .push(Instruction::ChangeWeather(ChangeWeather {
                        new_weather: Weather::SUN,
                        new_weather_turns_remaining: WEATHER_ABILITY_TURNS,
                        previous_weather: state.weather.weather_type,
                        previous_weather_turns_remaining: state.weather.turns_remaining,
                    }));
                state.weather.weather_type = Weather::SUN;
                state.weather.turns_remaining = WEATHER_ABILITY_TURNS;
            }
        }
        Abilities::DESOLATELAND => {
            if weather_should_be_set(state, Weather::HARSHSUN) {
                instructions
                    .instruction_list
                    .push(Instruction::ChangeWeather(ChangeWeather {
                        new_weather: Weather::HARSHSUN,
                        new_weather_turns_remaining: -1,
                        previous_weather: state.weather.weather_type,
                        previous_weather_turns_remaining: state.weather.turns_remaining,
                    }));
                state.weather.weather_type = Weather::HARSHSUN;
                state.weather.turns_remaining = -1;
            }
        }
        Abilities::MISTYSURGE => {
            if terrain_should_be_set(state, Terrain::MISTYTERRAIN) {
                instructions
                    .instruction_list
                    .push(Instruction::ChangeTerrain(ChangeTerrain {
                        new_terrain: Terrain::MISTYTERRAIN,
                        new_terrain_turns_remaining: 5,
                        previous_terrain: state.terrain.terrain_type,
                        previous_terrain_turns_remaining: state.terrain.turns_remaining,
                    }));
                state.terrain.terrain_type = Terrain::MISTYTERRAIN;
                state.terrain.turns_remaining = 5;
            }
        }
        Abilities::SANDSTREAM => {
            if regular_weather_should_be_set(state, Weather::SAND) {
                instructions
                    .instruction_list
                    .push(Instruction::ChangeWeather(ChangeWeather {
                        new_weather: Weather::SAND,
                        new_weather_turns_remaining: WEATHER_ABILITY_TURNS,
                        previous_weather: state.weather.weather_type,
                        previous_weather_turns_remaining: state.weather.turns_remaining,
                    }));
                state.weather.weather_type = Weather::SAND;
                state.weather.turns_remaining = WEATHER_ABILITY_TURNS;
            }
        }
        Abilities::INTIMIDATE => {
            let defender = defending_side.get_active_immutable();
            if !(defender_ability == Abilities::OWNTEMPO
                || defender_ability == Abilities::OBLIVIOUS
                || defender_ability == Abilities::INNERFOCUS
                || defender_ability == Abilities::SCRAPPY
                || defending_side
                    .volatile_statuses
                    .contains(&PokemonVolatileStatus::SUBSTITUTE))
            {
                if apply_boost_instruction_with_effective(
                    defending_side,
                    &PokemonBoostableStat::Attack,
                    &-1,
                    side_ref,
                    &side_ref.get_other_side(),
                    defender_ability,
                    defender_item_is_active,
                    instructions,
                ) {
                    let defender = defending_side.get_active_immutable();
                    if defender.item == Items::ADRENALINEORB && defender_item_is_active {
                        if apply_boost_instruction_with_effective(
                            defending_side,
                            &PokemonBoostableStat::Speed,
                            &1,
                            &side_ref.get_other_side(),
                            &side_ref.get_other_side(),
                            defender_ability,
                            defender_item_is_active,
                            instructions,
                        ) {
                            let adrenaline_orb_item_instruction =
                                Instruction::ChangeItem(ChangeItemInstruction {
                                    side_ref: side_ref.get_other_side(),
                                    current_item: Items::ADRENALINEORB,
                                    new_item: Items::NONE,
                                });
                            state.apply_one_instruction(&adrenaline_orb_item_instruction);
                            instructions
                                .instruction_list
                                .push(adrenaline_orb_item_instruction)
                        }
                    }
                }
            }
        }
        Abilities::DAUNTLESSSHIELD => {
            #[cfg(feature = "gen9")]
            if !active_pkmn.shield_boost_used {
                instructions
                    .instruction_list
                    .push(Instruction::ToggleShieldBoostUsed(
                        ToggleAbilityOnStartFlagInstruction {
                            side_ref: *side_ref,
                            pokemon_index: active_index,
                        },
                    ));
                active_pkmn.shield_boost_used = true;
                apply_boost_instruction_with_effective(
                    attacking_side,
                    &PokemonBoostableStat::Defense,
                    &1,
                    side_ref,
                    side_ref,
                    active_ability,
                    active_item_is_active,
                    instructions,
                );
            }
            #[cfg(not(feature = "gen9"))]
            {
                apply_boost_instruction_with_effective(
                    attacking_side,
                    &PokemonBoostableStat::Defense,
                    &1,
                    side_ref,
                    side_ref,
                    active_ability,
                    active_item_is_active,
                    instructions,
                );
            }
        }
        Abilities::GRASSYSURGE => {
            if terrain_should_be_set(state, Terrain::GRASSYTERRAIN) {
                instructions
                    .instruction_list
                    .push(Instruction::ChangeTerrain(ChangeTerrain {
                        new_terrain: Terrain::GRASSYTERRAIN,
                        new_terrain_turns_remaining: 5,
                        previous_terrain: state.terrain.terrain_type,
                        previous_terrain_turns_remaining: state.terrain.turns_remaining,
                    }));
                state.terrain.terrain_type = Terrain::GRASSYTERRAIN;
                state.terrain.turns_remaining = 5;
            }
        }
        Abilities::ELECTRICSURGE | Abilities::HADRONENGINE => {
            if terrain_should_be_set(state, Terrain::ELECTRICTERRAIN) {
                instructions
                    .instruction_list
                    .push(Instruction::ChangeTerrain(ChangeTerrain {
                        new_terrain: Terrain::ELECTRICTERRAIN,
                        new_terrain_turns_remaining: 5,
                        previous_terrain: state.terrain.terrain_type,
                        previous_terrain_turns_remaining: state.terrain.turns_remaining,
                    }));
                state.terrain.terrain_type = Terrain::ELECTRICTERRAIN;
                state.terrain.turns_remaining = 5;
            }
        }
        Abilities::DOWNLOAD => {
            if defending_side.calculate_boosted_stat(PokemonBoostableStat::Defense)
                < defending_side.calculate_boosted_stat(PokemonBoostableStat::SpecialDefense)
            {
                apply_boost_instruction_with_effective(
                    attacking_side,
                    &PokemonBoostableStat::Attack,
                    &1,
                    side_ref,
                    side_ref,
                    active_ability,
                    active_item_is_active,
                    instructions,
                );
            } else {
                apply_boost_instruction_with_effective(
                    attacking_side,
                    &PokemonBoostableStat::SpecialAttack,
                    &1,
                    side_ref,
                    side_ref,
                    active_ability,
                    active_item_is_active,
                    instructions,
                );
            }
        }
        Abilities::PRIMORDIALSEA => {
            if weather_should_be_set(state, Weather::HEAVYRAIN) {
                instructions
                    .instruction_list
                    .push(Instruction::ChangeWeather(ChangeWeather {
                        new_weather: Weather::HEAVYRAIN,
                        new_weather_turns_remaining: -1,
                        previous_weather: state.weather.weather_type,
                        previous_weather_turns_remaining: state.weather.turns_remaining,
                    }));
                state.weather.weather_type = Weather::HEAVYRAIN;
                state.weather.turns_remaining = -1;
            }
        }
        Abilities::SCREENCLEANER => {
            if state.side_one.side_conditions.reflect > 0 {
                instructions
                    .instruction_list
                    .push(Instruction::ChangeSideCondition(
                        ChangeSideConditionInstruction {
                            side_ref: SideReference::SideOne,
                            side_condition: PokemonSideCondition::Reflect,
                            amount: -1 * state.side_one.side_conditions.reflect,
                        },
                    ));
                state.side_one.side_conditions.reflect = 0;
            }
            if state.side_two.side_conditions.reflect > 0 {
                instructions
                    .instruction_list
                    .push(Instruction::ChangeSideCondition(
                        ChangeSideConditionInstruction {
                            side_ref: SideReference::SideTwo,
                            side_condition: PokemonSideCondition::Reflect,
                            amount: -1 * state.side_two.side_conditions.reflect,
                        },
                    ));
                state.side_two.side_conditions.reflect = 0;
            }
            if state.side_one.side_conditions.light_screen > 0 {
                instructions
                    .instruction_list
                    .push(Instruction::ChangeSideCondition(
                        ChangeSideConditionInstruction {
                            side_ref: SideReference::SideOne,
                            side_condition: PokemonSideCondition::LightScreen,
                            amount: -1 * state.side_one.side_conditions.light_screen,
                        },
                    ));
                state.side_one.side_conditions.light_screen = 0;
            }
            if state.side_two.side_conditions.light_screen > 0 {
                instructions
                    .instruction_list
                    .push(Instruction::ChangeSideCondition(
                        ChangeSideConditionInstruction {
                            side_ref: SideReference::SideTwo,
                            side_condition: PokemonSideCondition::LightScreen,
                            amount: -1 * state.side_two.side_conditions.light_screen,
                        },
                    ));
                state.side_two.side_conditions.light_screen = 0;
            }
            if state.side_one.side_conditions.aurora_veil > 0 {
                instructions
                    .instruction_list
                    .push(Instruction::ChangeSideCondition(
                        ChangeSideConditionInstruction {
                            side_ref: SideReference::SideOne,
                            side_condition: PokemonSideCondition::AuroraVeil,
                            amount: -1 * state.side_one.side_conditions.aurora_veil,
                        },
                    ));
                state.side_one.side_conditions.aurora_veil = 0;
            }
            if state.side_two.side_conditions.aurora_veil > 0 {
                instructions
                    .instruction_list
                    .push(Instruction::ChangeSideCondition(
                        ChangeSideConditionInstruction {
                            side_ref: SideReference::SideTwo,
                            side_condition: PokemonSideCondition::AuroraVeil,
                            amount: -1 * state.side_two.side_conditions.aurora_veil,
                        },
                    ));
                state.side_two.side_conditions.aurora_veil = 0;
            }
        }
        Abilities::SNOWWARNING => {
            #[cfg(feature = "gen9")]
            let weather_type = Weather::SNOW;
            #[cfg(not(feature = "gen9"))]
            let weather_type = Weather::HAIL;

            if regular_weather_should_be_set(state, weather_type) {
                instructions
                    .instruction_list
                    .push(Instruction::ChangeWeather(ChangeWeather {
                        new_weather: weather_type,
                        new_weather_turns_remaining: WEATHER_ABILITY_TURNS,
                        previous_weather: state.weather.weather_type,
                        previous_weather_turns_remaining: state.weather.turns_remaining,
                    }));
                state.weather.weather_type = weather_type;
                state.weather.turns_remaining = WEATHER_ABILITY_TURNS;
            }
        }
        Abilities::PSYCHICSURGE => {
            if terrain_should_be_set(state, Terrain::PSYCHICTERRAIN) {
                instructions
                    .instruction_list
                    .push(Instruction::ChangeTerrain(ChangeTerrain {
                        new_terrain: Terrain::PSYCHICTERRAIN,
                        new_terrain_turns_remaining: 5,
                        previous_terrain: state.terrain.terrain_type,
                        previous_terrain_turns_remaining: state.terrain.turns_remaining,
                    }));
                state.terrain.terrain_type = Terrain::PSYCHICTERRAIN;
                state.terrain.turns_remaining = 5;
            }
        }
        Abilities::DRIZZLE => {
            if regular_weather_should_be_set(state, Weather::RAIN) {
                instructions
                    .instruction_list
                    .push(Instruction::ChangeWeather(ChangeWeather {
                        new_weather: Weather::RAIN,
                        new_weather_turns_remaining: WEATHER_ABILITY_TURNS,
                        previous_weather: state.weather.weather_type,
                        previous_weather_turns_remaining: state.weather.turns_remaining,
                    }));
                state.weather.weather_type = Weather::RAIN;
                state.weather.turns_remaining = WEATHER_ABILITY_TURNS;
            }
        }
        _ => {}
    }
}

pub fn ability_on_switch_in(
    state: &mut State,
    side_ref: &SideReference,
    instructions: &mut StateInstructions,
) {
    ability_on_start(state, side_ref, instructions);
}

pub fn ability_modify_attack_being_used(
    state: &State,
    attacker_choice: &mut Choice,
    defender_choice: &Choice,
    attacking_side_ref: &SideReference,
) {
    let defending_side_ref = attacking_side_ref.get_other_side();
    let defending_ability = state.active_ability(&defending_side_ref);
    let (attacking_side, defending_side) = state.get_both_sides_immutable(attacking_side_ref);
    let attacking_pkmn = attacking_side.get_active_immutable();
    if state.active_ability_is_suppressed(attacking_side_ref) {
        return;
    }
    match attacking_pkmn.ability {
        #[cfg(any(feature = "gen9", feature = "gen8", feature = "gen7"))]
        Abilities::PRANKSTER => {
            if attacker_choice.category == MoveCategory::Status
                && defending_side
                    .get_active_immutable()
                    .has_type(&PokemonType::DARK)
            {
                attacker_choice.remove_all_effects();
            }
        }
        Abilities::BEADSOFRUIN => {
            if attacker_choice.category == MoveCategory::Special {
                attacker_choice.base_power *= 1.33;
            }
        }
        Abilities::SWORDOFRUIN => {
            if attacker_choice.category == MoveCategory::Physical {
                attacker_choice.base_power *= 1.33;
            }
        }
        Abilities::SHARPNESS => {
            if attacker_choice.flags.slicing {
                attacker_choice.base_power *= 1.5;
            }
        }
        Abilities::WATERBUBBLE => {
            if attacker_choice.move_type == PokemonType::WATER {
                attacker_choice.base_power *= 2.0;
            }
        }
        Abilities::DRAGONSMAW => {
            if attacker_choice.move_type == PokemonType::DRAGON {
                attacker_choice.base_power *= 1.5;
            }
        }
        Abilities::HADRONENGINE => {
            if attacker_choice.category == MoveCategory::Special
                && state.terrain_is_active(&Terrain::ELECTRICTERRAIN)
            {
                attacker_choice.base_power *= 1.33;
            }
        }
        Abilities::ORICHALCUMPULSE => {
            if attacker_choice.category == MoveCategory::Physical
                && (state.weather_is_active(&Weather::SUN)
                    || state.weather_is_active(&Weather::HARSHSUN))
            {
                attacker_choice.base_power *= 1.33;
            }
        }
        Abilities::GALVANIZE => {
            if attacker_choice.move_type == PokemonType::NORMAL {
                attacker_choice.move_type = PokemonType::ELECTRIC;
                attacker_choice.base_power *= 1.2;
            }
        }
        Abilities::DRAGONIZE => {
            if attacker_choice.move_type == PokemonType::NORMAL {
                attacker_choice.move_type = PokemonType::DRAGON;
                attacker_choice.base_power *= 1.2;
            }
        }
        #[cfg(any(feature = "gen9", feature = "gen8", feature = "gen7"))]
        Abilities::AERILATE => {
            if attacker_choice.move_type == PokemonType::NORMAL {
                attacker_choice.move_type = PokemonType::FLYING;
                attacker_choice.base_power *= 1.2;
            }
        }
        #[cfg(any(feature = "gen6", feature = "gen5", feature = "gen4"))]
        Abilities::AERILATE => {
            if attacker_choice.move_type == PokemonType::NORMAL {
                attacker_choice.move_type = PokemonType::FLYING;
                attacker_choice.base_power *= 1.3;
            }
        }
        Abilities::NEUROFORCE => {
            if type_effectiveness_modifier(
                &attacker_choice.move_type,
                &defending_side.get_active_immutable(),
            ) > 1.0
            {
                attacker_choice.base_power *= 1.25;
            }
        }
        Abilities::STAKEOUT => {
            if defender_choice.category == MoveCategory::Switch {
                attacker_choice.base_power *= 2.0;
            }
        }
        Abilities::TECHNICIAN => {
            if attacker_choice.base_power <= 60.0 {
                attacker_choice.base_power *= 1.5;
            }
        }
        #[cfg(any(feature = "gen9", feature = "gen8", feature = "gen7"))]
        Abilities::REFRIGERATE => {
            if attacker_choice.move_type == PokemonType::NORMAL {
                attacker_choice.move_type = PokemonType::ICE;
                attacker_choice.base_power *= 1.2;
            }
        }
        #[cfg(any(feature = "gen6", feature = "gen5", feature = "gen4"))]
        Abilities::REFRIGERATE => {
            if attacker_choice.move_type == PokemonType::NORMAL {
                attacker_choice.move_type = PokemonType::ICE;
                attacker_choice.base_power *= 1.3;
            }
        }
        Abilities::SUPREMEOVERLORD => {
            let mut boost_amount = 1.0;
            boost_amount += 0.1 * attacking_side.num_fainted_pkmn() as f32;
            attacker_choice.base_power *= boost_amount;
        }
        Abilities::ADAPTABILITY => {
            if attacking_pkmn.has_type(&attacker_choice.move_type) {
                if attacking_pkmn.terastallized
                    && attacker_choice.move_type == attacking_pkmn.tera_type
                    && (attacking_pkmn.types.0 == attacker_choice.move_type
                        || attacking_pkmn.types.1 == attacker_choice.move_type)
                {
                    attacker_choice.base_power *= 2.25 / 2.0;
                } else {
                    attacker_choice.base_power *= 2.0 / 1.5;
                }
            }
        }
        Abilities::LONGREACH => {
            attacker_choice.flags.contact = false;
        }
        Abilities::PUREPOWER => {
            if attacker_choice.category == MoveCategory::Physical {
                attacker_choice.base_power *= 2.0;
            }
        }
        Abilities::TINTEDLENS => {
            if type_effectiveness_modifier(
                &attacker_choice.move_type,
                &defending_side.get_active_immutable(),
            ) < 1.0
            {
                attacker_choice.base_power *= 2.0;
            }
        }
        Abilities::FLAREBOOST => {
            if attacking_pkmn.status == PokemonStatus::BURN {
                attacker_choice.base_power *= 1.5;
            }
        }
        Abilities::LIQUIDVOICE => {
            if attacker_choice.flags.sound {
                attacker_choice.move_type = PokemonType::WATER;
            }
        }
        Abilities::NOGUARD => attacker_choice.accuracy = 100.0,
        Abilities::TORRENT => {
            if attacker_choice.move_type == PokemonType::WATER
                && attacking_pkmn.hp <= attacking_pkmn.maxhp / 3
            {
                attacker_choice.base_power *= 1.5;
            }
        }
        Abilities::SERENEGRACE => {
            if let Some(secondaries) = &mut attacker_choice.secondaries {
                for secondary in secondaries.iter_mut() {
                    secondary.chance *= 2.0;
                }
            }
        }
        Abilities::TOUGHCLAWS => {
            if attacker_choice.flags.contact {
                attacker_choice.base_power *= 1.3;
            }
        }
        Abilities::RECKLESS => {
            if attacker_choice.crash.is_some() || attacker_choice.recoil.is_some() {
                attacker_choice.base_power *= 1.2;
            }
        }
        Abilities::HUGEPOWER => {
            if attacker_choice.category == MoveCategory::Physical {
                attacker_choice.base_power *= 2.0;
            }
        }
        Abilities::SOLARPOWER => {
            if state.weather_is_active(&Weather::SUN) || state.weather_is_active(&Weather::HARSHSUN)
            {
                attacker_choice.base_power *= 1.5;
            }
        }
        Abilities::FAIRYAURA => {
            if attacker_choice.move_type == PokemonType::FAIRY
                && defending_ability != Abilities::AURABREAK
            {
                attacker_choice.base_power *= 1.33;
            }
        }
        Abilities::NORMALIZE => {
            attacker_choice.move_type = PokemonType::NORMAL;
        }
        Abilities::DARKAURA => {
            if attacker_choice.move_type == PokemonType::DARK
                && defending_ability != Abilities::AURABREAK
            {
                attacker_choice.base_power *= 1.33;
            }
        }
        Abilities::VICTORYSTAR => {
            attacker_choice.accuracy *= 1.1;
        }
        Abilities::COMPOUNDEYES => {
            attacker_choice.accuracy *= 1.3;
        }
        Abilities::STEELWORKER | Abilities::STEELYSPIRIT => {
            if attacker_choice.move_type == PokemonType::STEEL {
                attacker_choice.base_power *= 1.5;
            }
        }
        #[cfg(any(
            feature = "gen8",
            feature = "gen7",
            feature = "gen6",
            feature = "gen5",
            feature = "gen4"
        ))]
        Abilities::TRANSISTOR => {
            if attacker_choice.move_type == PokemonType::ELECTRIC {
                attacker_choice.base_power *= 1.5;
            }
        }
        #[cfg(any(feature = "gen9"))]
        Abilities::TRANSISTOR => {
            if attacker_choice.move_type == PokemonType::ELECTRIC {
                attacker_choice.base_power *= 1.3;
            }
        }
        Abilities::STENCH => {
            let mut already_flinches = false;
            if let Some(secondaries) = &mut attacker_choice.secondaries {
                for secondary in secondaries.iter() {
                    if secondary.effect == Effect::VolatileStatus(PokemonVolatileStatus::FLINCH) {
                        already_flinches = true;
                    }
                }
            }
            if !already_flinches {
                attacker_choice.add_or_create_secondaries(Secondary {
                    chance: 10.0,
                    target: MoveTarget::Opponent,
                    effect: Effect::VolatileStatus(PokemonVolatileStatus::FLINCH),
                })
            }
        }
        Abilities::SWARM => {
            if attacker_choice.move_type == PokemonType::BUG
                && attacking_pkmn.hp <= attacking_pkmn.maxhp / 3
            {
                attacker_choice.base_power *= 1.5;
            }
        }
        Abilities::GORILLATACTICS => {
            if attacker_choice.category == MoveCategory::Physical {
                attacker_choice.base_power *= 1.5;
            }
        }
        Abilities::BLAZE => {
            if attacker_choice.move_type == PokemonType::FIRE
                && attacking_pkmn.hp <= attacking_pkmn.maxhp / 3
            {
                attacker_choice.base_power *= 1.5;
            }
        }
        Abilities::OVERGROW => {
            if attacker_choice.move_type == PokemonType::GRASS
                && attacking_pkmn.hp <= attacking_pkmn.maxhp / 3
            {
                attacker_choice.base_power *= 1.5;
            }
        }
        Abilities::ANALYTIC => {
            if !attacker_choice.first_move {
                attacker_choice.base_power *= 1.3;
            }
        }
        Abilities::MEGALAUNCHER => {
            if attacker_choice.flags.pulse {
                attacker_choice.base_power *= 1.5;
            };
        }
        #[cfg(any(feature = "gen9", feature = "gen8", feature = "gen7"))]
        Abilities::PIXILATE => {
            if attacker_choice.move_type == PokemonType::NORMAL {
                attacker_choice.move_type = PokemonType::FAIRY;
                attacker_choice.base_power *= 1.2;
            }
        }
        #[cfg(any(feature = "gen6", feature = "gen5", feature = "gen4"))]
        Abilities::PIXILATE => {
            if attacker_choice.move_type == PokemonType::NORMAL {
                attacker_choice.move_type = PokemonType::FAIRY;
                attacker_choice.base_power *= 1.3;
            }
        }
        Abilities::DEFEATIST => {
            if attacking_pkmn.hp <= attacking_pkmn.maxhp / 2 {
                attacker_choice.base_power *= 0.5;
            }
        }
        Abilities::ROCKYPAYLOAD => {
            if attacker_choice.move_type == PokemonType::ROCK {
                attacker_choice.base_power *= 1.5;
            }
        }
        Abilities::PUNKROCK => {
            if attacker_choice.flags.sound {
                attacker_choice.base_power *= 1.3;
            }
        }
        Abilities::STRONGJAW => {
            if attacker_choice.flags.bite {
                attacker_choice.base_power *= 1.5;
            }
        }
        Abilities::BATTERY => {
            if attacker_choice.category == MoveCategory::Special {
                attacker_choice.base_power *= 1.3;
            }
        }
        Abilities::SHEERFORCE => {
            let mut sheer_force_volatile_boosted = false;
            if let Some(attacker_volatile_status) = &attacker_choice.volatile_status {
                let is_soak_typechange_marker = attacker_choice.move_id == Choices::SOAK
                    && attacker_volatile_status.volatile_status
                        == PokemonVolatileStatus::TYPECHANGE;
                if attacker_volatile_status.volatile_status
                    != PokemonVolatileStatus::PARTIALLYTRAPPED
                    && attacker_volatile_status.volatile_status != PokemonVolatileStatus::LOCKEDMOVE
                    && attacker_volatile_status.volatile_status != PokemonVolatileStatus::SMACKDOWN
                    && !is_soak_typechange_marker
                {
                    sheer_force_volatile_boosted = true;
                }
            }
            if attacker_choice.secondaries.is_some() || sheer_force_volatile_boosted {
                attacker_choice.base_power *= 1.3;
                attacker_choice.secondaries = None;
                attacker_choice.volatile_status = None
            }
        }
        Abilities::IRONFIST => {
            if attacker_choice.flags.punch {
                attacker_choice.base_power *= 1.2;
            }
        }
        Abilities::UNSEENFIST => {
            if attacker_choice.flags.contact {
                attacker_choice.bypasses_protect = true;
                attacker_choice.protect_bypass_damage_multiplier = 0.25;
            }
        }
        Abilities::PIERCINGDRILL => {
            if attacker_choice.flags.contact {
                attacker_choice.bypasses_protect = true;
                attacker_choice.protect_bypass_damage_multiplier = 0.25;
            }
        }
        Abilities::HUSTLE => {
            if attacker_choice.category == MoveCategory::Physical {
                attacker_choice.base_power *= 1.5;
                attacker_choice.accuracy *= 0.80
            }
        }
        Abilities::POISONTOUCH => {
            if attacker_choice.flags.contact {
                attacker_choice.add_or_create_secondaries(Secondary {
                    chance: 30.0,
                    target: MoveTarget::Opponent,
                    effect: Effect::Status(PokemonStatus::POISON),
                })
            }
        }
        Abilities::TOXICCHAIN => {
            if attacker_choice.target == MoveTarget::Opponent {
                attacker_choice.add_or_create_secondaries(Secondary {
                    chance: 30.0,
                    target: MoveTarget::Opponent,
                    effect: Effect::Status(PokemonStatus::TOXIC),
                })
            }
        }
        Abilities::GUTS => {
            if attacking_pkmn.status != PokemonStatus::NONE {
                attacker_choice.base_power *= 1.5;

                // not the right place to put this, but good enough
                if attacking_pkmn.status == PokemonStatus::BURN
                    && attacker_choice.category == MoveCategory::Physical
                {
                    attacker_choice.base_power *= 2.0;
                }
            }
        }
        Abilities::SANDFORCE => {
            if state.weather_is_active(&Weather::SAND)
                && (attacker_choice.move_type == PokemonType::ROCK
                    || attacker_choice.move_type == PokemonType::GROUND
                    || attacker_choice.move_type == PokemonType::STEEL)
            {
                attacker_choice.base_power *= 1.3;
            }
        }
        Abilities::TOXICBOOST => {
            if attacking_pkmn.status == PokemonStatus::POISON
                || attacking_pkmn.status == PokemonStatus::TOXIC
            {
                attacker_choice.base_power *= 1.5;
            }
        }
        _ => {}
    }
}

pub fn ability_modify_attack_against(
    state: &State,
    attacker_choice: &mut Choice,
    defender_choice: &Choice,
    attacking_side_ref: &SideReference,
) {
    let target_side_ref = attacking_side_ref.get_other_side();
    let attacking_ability_is_suppressed = state.active_ability_is_suppressed(attacking_side_ref);
    let target_ability_is_suppressed = state.active_ability_is_suppressed(&target_side_ref);
    let (attacking_side, defending_side) = state.get_both_sides_immutable(attacking_side_ref);
    let attacking_pkmn = attacking_side.get_active_immutable();
    let target_pkmn = defending_side.get_active_immutable();
    if attacker_choice.target == MoveTarget::User {
        return;
    }
    let attacking_ability = if attacking_ability_is_suppressed {
        Abilities::NONE
    } else {
        attacking_pkmn.ability
    };
    let target_ability = if target_ability_is_suppressed {
        Abilities::NONE
    } else {
        target_pkmn.ability
    };
    if (attacking_ability == Abilities::MOLDBREAKER
        || attacker_choice.move_id == Choices::MOONGEISTBEAM
        || attacker_choice.move_id == Choices::PHOTONGEYSER
        || attacker_choice.move_id == Choices::SUNSTEELSTRIKE
        || (attacking_ability == Abilities::MYCELIUMMIGHT
            && attacker_choice.category == MoveCategory::Status)
        || attacking_ability == Abilities::TERAVOLT
        || attacking_ability == Abilities::TURBOBLAZE)
        && state.active_ignores_target_ability(
            attacking_side_ref,
            &attacker_choice.move_id,
            target_ability,
            attacker_choice.category == MoveCategory::Status,
        )
        && mold_breaker_ignores(&target_ability)
    {
        return;
    }

    match target_ability {
        Abilities::TABLETSOFRUIN => {
            if attacker_choice.category == MoveCategory::Physical {
                attacker_choice.base_power *= 0.75;
            }
        }
        Abilities::VESSELOFRUIN => {
            if attacker_choice.category == MoveCategory::Special {
                attacker_choice.base_power *= 0.75;
            }
        }
        Abilities::ARMORTAIL => {
            if attacker_choice.priority > 0 && attacker_choice.category != MoveCategory::Status {
                attacker_choice.remove_all_effects();
            }
        }
        Abilities::SOUNDPROOF => {
            if attacker_choice.flags.sound {
                attacker_choice.remove_all_effects();
                attacker_choice.accuracy = 0.0;
            }
        }
        Abilities::POISONPOINT => {
            if attacker_choice.flags.contact {
                attacker_choice.add_or_create_secondaries(Secondary {
                    chance: 33.0,
                    target: MoveTarget::User,
                    effect: Effect::Status(PokemonStatus::POISON),
                })
            }
        }
        Abilities::BULLETPROOF => {
            if attacker_choice.flags.bullet {
                attacker_choice.remove_all_effects();
                attacker_choice.accuracy = 0.0;
            }
        }
        Abilities::MULTISCALE => {
            if target_pkmn.hp == target_pkmn.maxhp {
                attacker_choice.base_power /= 2.0;
            }
        }
        Abilities::LIGHTNINGROD => {
            if attacker_choice.move_type == PokemonType::ELECTRIC {
                attacker_choice.remove_all_effects();
                attacker_choice.accuracy = 100.0;
                attacker_choice.target = MoveTarget::Opponent;
                attacker_choice.boost = Some(Boost {
                    boosts: StatBoosts {
                        attack: 0,
                        defense: 0,
                        special_attack: 1,
                        special_defense: 0,
                        speed: 0,
                        accuracy: 0,
                    },
                    target: MoveTarget::Opponent,
                });
                attacker_choice.category = MoveCategory::Status;
            }
        }
        Abilities::EARTHEATER => {
            if attacker_choice.move_type == PokemonType::GROUND {
                attacker_choice.remove_all_effects();
                attacker_choice.base_power = 0.0;
                attacker_choice.heal = Some(Heal {
                    target: MoveTarget::Opponent,
                    amount: 0.25,
                });
                attacker_choice.category = MoveCategory::Status;
            }
        }
        Abilities::STEAMENGINE => {
            if attacker_choice.move_type == PokemonType::WATER
                || attacker_choice.move_type == PokemonType::FIRE
            {
                attacker_choice.add_or_create_secondaries(Secondary {
                    chance: 100.0,
                    target: MoveTarget::Opponent,
                    effect: Effect::Boost(StatBoosts {
                        attack: 0,
                        defense: 0,
                        special_attack: 0,
                        special_defense: 0,
                        speed: 6,
                        accuracy: 0,
                    }),
                });
            }
        }
        Abilities::THERMALEXCHANGE => {
            if attacker_choice.move_type == PokemonType::FIRE {
                attacker_choice.add_or_create_secondaries(Secondary {
                    chance: 100.0,
                    target: MoveTarget::Opponent,
                    effect: Effect::Boost(StatBoosts {
                        attack: 1,
                        defense: 0,
                        special_attack: 0,
                        special_defense: 0,
                        speed: 0,
                        accuracy: 0,
                    }),
                });
            }
        }
        #[cfg(any(feature = "gen9", feature = "gen8", feature = "gen7"))]
        Abilities::WEAKARMOR => {
            if attacker_choice.category == MoveCategory::Physical {
                attacker_choice.add_or_create_secondaries(Secondary {
                    chance: 100.0,
                    target: MoveTarget::Opponent,
                    effect: Effect::Boost(StatBoosts {
                        attack: 0,
                        defense: -1,
                        special_attack: 0,
                        special_defense: 0,
                        speed: 2,
                        accuracy: 0,
                    }),
                });
            }
        }
        #[cfg(any(feature = "gen6", feature = "gen5", feature = "gen4"))]
        Abilities::WEAKARMOR => {
            if attacker_choice.category == MoveCategory::Physical {
                attacker_choice.add_or_create_secondaries(Secondary {
                    chance: 100.0,
                    target: MoveTarget::Opponent,
                    effect: Effect::Boost(StatBoosts {
                        attack: 0,
                        defense: -1,
                        special_attack: 0,
                        special_defense: 0,
                        speed: 1,
                        accuracy: 0,
                    }),
                });
            }
        }
        Abilities::QUEENLYMAJESTY => {
            if attacker_choice.priority > 0 {
                attacker_choice.remove_all_effects();
                attacker_choice.accuracy = 0.0;
            }
        }
        Abilities::SAPSIPPER => {
            if attacker_choice.move_type == PokemonType::GRASS {
                attacker_choice.remove_all_effects();
                attacker_choice.accuracy = 100.0;
                attacker_choice.target = MoveTarget::Opponent;
                attacker_choice.boost = Some(Boost {
                    boosts: StatBoosts {
                        attack: 1,
                        defense: 0,
                        special_attack: 0,
                        special_defense: 0,
                        speed: 0,
                        accuracy: 0,
                    },
                    target: MoveTarget::Opponent,
                });
                attacker_choice.category = MoveCategory::Status;
            }
        }
        Abilities::SHADOWSHIELD => {
            if target_pkmn.hp == target_pkmn.maxhp {
                attacker_choice.base_power /= 2.0;
            }
        }
        Abilities::NOGUARD => {
            attacker_choice.accuracy = 100.0;
        }
        Abilities::MARVELSCALE => {
            if target_pkmn.status != PokemonStatus::NONE
                && attacker_choice.category == MoveCategory::Physical
            {
                attacker_choice.base_power /= 1.5;
            }
        }
        #[cfg(feature = "gen3")]
        Abilities::EFFECTSPORE => {
            if attacker_choice.flags.contact {
                attacker_choice.add_or_create_secondaries(Secondary {
                    chance: 3.30,
                    target: MoveTarget::User,
                    effect: Effect::Status(PokemonStatus::POISON),
                });
                attacker_choice.add_or_create_secondaries(Secondary {
                    chance: 3.30,
                    target: MoveTarget::User,
                    effect: Effect::Status(PokemonStatus::PARALYZE),
                });
                attacker_choice.add_or_create_secondaries(Secondary {
                    chance: 3.30,
                    target: MoveTarget::User,
                    effect: Effect::Status(PokemonStatus::SLEEP),
                });
            }
        }

        #[cfg(not(feature = "gen3"))]
        Abilities::EFFECTSPORE => {
            if attacker_choice.flags.contact {
                attacker_choice.add_or_create_secondaries(Secondary {
                    chance: 9.0,
                    target: MoveTarget::User,
                    effect: Effect::Status(PokemonStatus::POISON),
                });
                attacker_choice.add_or_create_secondaries(Secondary {
                    chance: 10.0,
                    target: MoveTarget::User,
                    effect: Effect::Status(PokemonStatus::PARALYZE),
                });
                attacker_choice.add_or_create_secondaries(Secondary {
                    chance: 11.0,
                    target: MoveTarget::User,
                    effect: Effect::Status(PokemonStatus::SLEEP),
                });
            }
        }
        Abilities::FLAMEBODY => {
            if attacker_choice.flags.contact {
                attacker_choice.add_or_create_secondaries(Secondary {
                    chance: 30.0,
                    target: MoveTarget::User,
                    effect: Effect::Status(PokemonStatus::BURN),
                });
            }
        }
        Abilities::GOOEY => {
            if attacker_choice.flags.contact {
                attacker_choice.add_or_create_secondaries(Secondary {
                    chance: 100.0,
                    target: MoveTarget::User,
                    effect: Effect::Boost(StatBoosts {
                        attack: 0,
                        defense: 0,
                        special_attack: 0,
                        special_defense: 0,
                        speed: -1,
                        accuracy: 0,
                    }),
                })
            }
        }
        Abilities::MOTORDRIVE => {
            if attacker_choice.move_type == PokemonType::ELECTRIC {
                attacker_choice.remove_all_effects();
                attacker_choice.accuracy = 100.0;
                attacker_choice.target = MoveTarget::Opponent;
                attacker_choice.boost = Some(Boost {
                    boosts: StatBoosts {
                        attack: 0,
                        defense: 0,
                        special_attack: 0,
                        special_defense: 0,
                        speed: 1,
                        accuracy: 0,
                    },
                    target: MoveTarget::Opponent,
                });
                attacker_choice.category = MoveCategory::Status;
            }
        }
        Abilities::WINDRIDER => {
            if attacker_choice.flags.wind {
                attacker_choice.remove_all_effects();
                attacker_choice.accuracy = 100.0;
                attacker_choice.target = MoveTarget::Opponent;
                attacker_choice.boost = Some(Boost {
                    boosts: StatBoosts {
                        attack: 1,
                        defense: 0,
                        special_attack: 0,
                        special_defense: 0,
                        speed: 0,
                        accuracy: 0,
                    },
                    target: MoveTarget::Opponent,
                });
                attacker_choice.category = MoveCategory::Status;
            }
        }
        Abilities::SUCTIONCUPS => {
            attacker_choice.flags.drag = false;
        }
        Abilities::WONDERGUARD => {
            if attacker_choice.category != MoveCategory::Status
                && type_effectiveness_modifier(&attacker_choice.move_type, &target_pkmn) <= 1.0
            {
                attacker_choice.remove_all_effects();
                attacker_choice.base_power = 0.0;
            }
        }
        Abilities::FAIRYAURA => {
            if attacker_choice.move_type == PokemonType::FAIRY {
                attacker_choice.base_power *= 1.33;
            }
        }
        Abilities::LEVITATE => {
            if attacker_choice.move_type == PokemonType::GROUND
                && attacker_choice.target == MoveTarget::Opponent
                && attacker_choice.move_id != Choices::THOUSANDARROWS
            {
                attacker_choice.base_power = 0.0;
            }
        }
        Abilities::STATIC => {
            if attacker_choice.flags.contact {
                attacker_choice.add_or_create_secondaries(Secondary {
                    chance: 30.0,
                    target: MoveTarget::User,
                    effect: Effect::Status(PokemonStatus::PARALYZE),
                })
            }
        }
        Abilities::WONDERSKIN => {
            if attacker_choice.category == MoveCategory::Status && attacker_choice.accuracy > 50.0 {
                attacker_choice.accuracy = 50.0;
            }
        }
        Abilities::THICKFAT => {
            if attacker_choice.move_type == PokemonType::FIRE
                || attacker_choice.move_type == PokemonType::ICE
            {
                attacker_choice.base_power /= 2.0;
            }
        }
        Abilities::FLASHFIRE => {
            if attacker_choice.move_type == PokemonType::FIRE {
                attacker_choice.remove_all_effects();
                attacker_choice.volatile_status = Some(VolatileStatus {
                    target: MoveTarget::Opponent,
                    volatile_status: PokemonVolatileStatus::FLASHFIRE,
                });
            }
        }
        Abilities::WELLBAKEDBODY => {
            if attacker_choice.move_type == PokemonType::FIRE {
                attacker_choice.remove_all_effects();
                attacker_choice.boost = Some(Boost {
                    boosts: StatBoosts {
                        attack: 0,
                        defense: 2,
                        special_attack: 0,
                        special_defense: 0,
                        speed: 0,
                        accuracy: 0,
                    },
                    target: MoveTarget::Opponent,
                });
            }
        }
        Abilities::DAZZLING => {
            if attacker_choice.priority > 0 {
                attacker_choice.accuracy = 0.0;
            }
        }
        Abilities::LIQUIDOOZE => {
            if let Some(drain) = attacker_choice.drain {
                attacker_choice.drain = Some(-1.0 * drain);
            }
        }
        Abilities::PRISMARMOR => {
            if type_effectiveness_modifier(&attacker_choice.move_type, &target_pkmn) > 1.0 {
                attacker_choice.base_power *= 0.75;
            }
        }
        Abilities::HEATPROOF => {
            if attacker_choice.move_type == PokemonType::FIRE {
                attacker_choice.base_power *= 0.5;
            }
        }
        Abilities::SHIELDDUST => {
            if let Some(secondaries) = &mut attacker_choice.secondaries {
                for secondary in secondaries.iter_mut() {
                    if secondary.target == MoveTarget::Opponent {
                        secondary.chance = 0.0;
                    }
                }
            }
        }
        Abilities::GRASSPELT => {
            if state.terrain_is_active(&Terrain::GRASSYTERRAIN)
                && attacker_choice.category == MoveCategory::Physical
            {
                attacker_choice.base_power /= 1.5;
            }
        }
        Abilities::FILTER => {
            if type_effectiveness_modifier(&attacker_choice.move_type, &target_pkmn) > 1.0 {
                attacker_choice.base_power *= 0.75;
            }
        }
        Abilities::FURCOAT => {
            if attacker_choice.category == MoveCategory::Physical {
                attacker_choice.base_power *= 0.5;
            }
        }
        Abilities::TANGLINGHAIR => {
            if attacker_choice.flags.contact {
                attacker_choice.add_or_create_secondaries(Secondary {
                    chance: 100.0,
                    target: MoveTarget::User,
                    effect: Effect::Boost(StatBoosts {
                        attack: 0,
                        defense: 0,
                        special_attack: 0,
                        special_defense: 0,
                        speed: -1,
                        accuracy: 0,
                    }),
                })
            }
        }
        Abilities::MAGICBOUNCE => {
            if attacker_choice.flags.reflectable {
                attacker_choice.target = MoveTarget::User;
                if let Some(side_condition) = &mut attacker_choice.side_condition {
                    if side_condition.target == MoveTarget::Opponent {
                        side_condition.target = MoveTarget::User;
                    }
                }
                if let Some(status) = &mut attacker_choice.status {
                    if status.target == MoveTarget::Opponent {
                        status.target = MoveTarget::User;
                    }
                }
                if let Some(volatile_status) = &mut attacker_choice.volatile_status {
                    if volatile_status.target == MoveTarget::Opponent {
                        volatile_status.target = MoveTarget::User;
                    }
                }
            }
        }
        Abilities::STORMDRAIN => {
            if attacker_choice.move_type == PokemonType::WATER {
                attacker_choice.remove_all_effects();
                attacker_choice.accuracy = 100.0;
                attacker_choice.target = MoveTarget::Opponent;
                attacker_choice.boost = Some(Boost {
                    boosts: StatBoosts {
                        attack: 0,
                        defense: 0,
                        special_attack: 1,
                        special_defense: 0,
                        speed: 0,
                        accuracy: 0,
                    },
                    target: MoveTarget::Opponent,
                });
                attacker_choice.category = MoveCategory::Status;
            }
        }
        Abilities::WATERCOMPACTION => {
            if attacker_choice.move_type == PokemonType::WATER {
                attacker_choice.add_or_create_secondaries(Secondary {
                    chance: 100.0,
                    target: MoveTarget::Opponent,
                    effect: Effect::Boost(StatBoosts {
                        attack: 0,
                        defense: 2,
                        special_attack: 0,
                        special_defense: 0,
                        speed: 0,
                        accuracy: 0,
                    }),
                });
            }
        }
        Abilities::JUSTIFIED => {
            if attacker_choice.move_type == PokemonType::DARK {
                attacker_choice.add_or_create_secondaries(Secondary {
                    chance: 100.0,
                    target: MoveTarget::Opponent,
                    effect: Effect::Boost(StatBoosts {
                        attack: 1,
                        defense: 0,
                        special_attack: 0,
                        special_defense: 0,
                        speed: 0,
                        accuracy: 0,
                    }),
                })
            }
        }
        Abilities::ICESCALES => {
            if attacker_choice.category == MoveCategory::Special {
                attacker_choice.base_power *= 0.5;
            }
        }
        Abilities::WATERABSORB => {
            if attacker_choice.move_type == PokemonType::WATER {
                attacker_choice.remove_all_effects();
                attacker_choice.base_power = 0.0;
                attacker_choice.heal = Some(Heal {
                    target: MoveTarget::Opponent,
                    amount: 0.25,
                });
                attacker_choice.category = MoveCategory::Status;
            }
        }
        Abilities::DRYSKIN => {
            if attacker_choice.move_type == PokemonType::WATER {
                attacker_choice.remove_all_effects();
                attacker_choice.base_power = 0.0;
                attacker_choice.heal = Some(Heal {
                    target: MoveTarget::Opponent,
                    amount: 0.25,
                });
                attacker_choice.category = MoveCategory::Status;
            } else if attacker_choice.move_type == PokemonType::FIRE {
                attacker_choice.base_power *= 1.25;
            }
        }
        Abilities::FLUFFY => {
            if attacker_choice.flags.contact {
                attacker_choice.base_power *= 0.5;
            }
            if attacker_choice.move_type == PokemonType::FIRE {
                attacker_choice.base_power *= 2.0;
            }
        }
        Abilities::PUNKROCK => {
            if attacker_choice.flags.sound {
                attacker_choice.base_power /= 2.0;
            }
        }
        Abilities::DAMP => {
            if [
                Choices::SELFDESTRUCT,
                Choices::EXPLOSION,
                Choices::MINDBLOWN,
                Choices::MISTYEXPLOSION,
            ]
            .contains(&attacker_choice.move_id)
            {
                attacker_choice.accuracy = 0.0;
                attacker_choice.heal = None;
            }
        }
        Abilities::VOLTABSORB => {
            #[cfg(feature = "gen3")]
            let activate = attacker_choice.move_type == PokemonType::ELECTRIC
                && attacker_choice.category != MoveCategory::Status;

            #[cfg(not(feature = "gen3"))]
            let activate = attacker_choice.move_type == PokemonType::ELECTRIC;

            if activate {
                attacker_choice.remove_all_effects();
                attacker_choice.accuracy = 100.0;
                attacker_choice.base_power = 0.0;
                attacker_choice.heal = Some(Heal {
                    target: MoveTarget::Opponent,
                    amount: 0.25,
                });
                attacker_choice.category = MoveCategory::Status;
            }
        }
        Abilities::SOLIDROCK => {
            if type_effectiveness_modifier(&attacker_choice.move_type, &target_pkmn) > 1.0 {
                attacker_choice.base_power *= 0.75;
            }
        }
        Abilities::OVERCOAT => {
            if attacker_choice.flags.powder {
                attacker_choice.remove_all_effects();
                attacker_choice.accuracy = 0.0
            }
        }
        Abilities::GOODASGOLD => {
            // This engine doesn't distinguish "targetting other pkmn" versus "targetting the side"
            // Thankfully it is a short list of moves that target the opponent side
            if attacker_choice.category == MoveCategory::Status
                && attacker_choice.target == MoveTarget::Opponent
                && ![
                    Choices::STEALTHROCK,
                    Choices::STICKYWEB,
                    Choices::TOXICSPIKES,
                    Choices::SPIKES,
                ]
                .contains(&attacker_choice.move_id)
            {
                attacker_choice.remove_all_effects();
            }
        }
        Abilities::RATTLED => {
            if attacker_choice.move_type == PokemonType::BUG
                || attacker_choice.move_type == PokemonType::DARK
                || attacker_choice.move_type == PokemonType::GHOST
            {
                attacker_choice.add_or_create_secondaries(Secondary {
                    chance: 100.0,
                    target: MoveTarget::Opponent,
                    effect: Effect::Boost(StatBoosts {
                        attack: 0,
                        defense: 0,
                        special_attack: 0,
                        special_defense: 0,
                        speed: 1,
                        accuracy: 0,
                    }),
                });
            }
        }
        Abilities::WATERBUBBLE => {
            if attacker_choice.move_type == PokemonType::FIRE {
                attacker_choice.base_power /= 2.0;
            }
        }
        Abilities::PURIFYINGSALT => {
            if attacker_choice.move_type == PokemonType::GHOST {
                attacker_choice.base_power /= 2.0;
            }
        }
        _ => {}
    }
}
