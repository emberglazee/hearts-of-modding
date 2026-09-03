#!/usr/bin/env python3
"""
Fix scope registrations for flagged entities in hoi4_data.json.

Two categories:
  1) "not found in vanilla analysis" (712) - entities the parser missed 
     but often have correct scopes from the wiki. Keep current where reasonable.
  2) "ambiguous scopes" (277) - entities found in multiple scopes that need 
     narrowing based on deep HOI4 modding knowledge.

Only update scope if 100% certain. False negative (entity has no scope, silently ignored) 
is better than false positive diagnostic.
"""

import json
import sys
from copy import deepcopy

JSON_PATH = "server/assets/hoi4_data.json"
FLAGGED_PATH = "server/scripts/flagged_ambiguous_entities.json"

ALL_SCOPES = ['Global', 'Country', 'State', 'Character', 'Unit']
# Note: Scope, Ace, StrategicRegion, Idea, FocusTree, NationalFocus, ModifierBag exist
# but only the 5 core are used in usage arrays.

def load_json(path):
    with open(path) as f:
        return json.load(f)

def save_json(path, data):
    with open(path, 'w') as f:
        json.dump(data, f, indent=2)
    print(f"Saved {path}")

def get_entity(data, category, name):
    """Get entity reference from data by category."""
    if category == 'triggers':
        return data.get('triggers', {}).get(name)
    elif category == 'effects':
        return data.get('effects', {}).get(name)
    elif category == 'modifiers':
        return data.get('modifiers', {}).get(name)
    return None

def set_scopes(entity, scopes):
    """Set the usage scopes on an entity."""
    if entity is None:
        return False
    if 'scopes' not in entity:
        entity['scopes'] = {'usage': scopes, 'usage_restriction': ''}
    else:
        # Preserve any existing usage_restriction
        restriction = entity['scopes'].get('usage_restriction', '')
        entity['scopes']['usage'] = scopes
        entity['scopes']['usage_restriction'] = restriction
    return True

def main():
    data = load_json(JSON_PATH)
    flagged = load_json(FLAGGED_PATH)
    
    # Track changes
    changes = []
    
    # ==============================================================
    # CATEGORY 1: Meta-keywords and flow control - ALL scopes
    # These are valid from any scope context
    # ==============================================================
    meta_all_scopes = {
        'triggers': [
            'AND', 'NOT', 'OR', 'if', 'hidden_trigger',
            'custom_override_tooltip', 'custom_trigger_tooltip',
            'count_triggers', 'meta_trigger', 'check_variable',
            'print_variables',
            # Career profile - meta/achievement, works everywhere
            'career_profile_check_medal', 'career_profile_check_playthrough_ratio',
            'career_profile_check_playthrough_value', 'career_profile_check_points',
            'career_profile_check_ratio', 'career_profile_check_ribbon',
            'career_profile_check_value',
            # Difficulty - works everywhere
            'difficulty',
            # Custom achievement - works everywhere  
            'has_completed_custom_achievement',
            'has_custom_difficulty_setting',
            'has_start_date',
            # Scope referentials - valid from any scope
            'FROM', 'ROOT', 'PREV',
            'owner', 'capital_scope', 'overlord',
        ],
        'effects': [
            'FROM', 'PREV', 'ROOT', 'owner', 'capital_scope', 'overlord',
            'custom_effect_tooltip', 'custom_override_tooltip',
            'effect_tooltip', 'character_list_tooltip',
            'scoped_sound_effect',
            'set_variable', 'set_temp_variable',
            'add_to_variable', 'subtract_from_variable', 'multiply_variable',
            'divide_variable', 'modulo_variable', 'clamp_variable',
            'add_to_array', 'remove_from_array', 'clear_array',
            'resize_array', 'find_highest_in_array', 'find_lowest_in_array',
            'get_highest_scored_country_temp', 'get_sorted_scored_countries_temp',
            'save_event_target_as',
            'set_global_flag', 'set_country_flag', 'set_character_flag',
            'modify_global_flag', 'modify_country_flag', 'modify_character_flag',
            'modify_state_flag', 'modify_unit_leader_flag', 'modify_mio_flag',
            'modify_project_flag',
            'has_completed_custom_achievement',
            'has_custom_difficulty_setting',
            'goto_province', 'force_update_map_mode',
            'mark_technology_tree_layout_dirty',
            'complete_prototype_reward_option',
        ],
    }
    
    for cat, names in meta_all_scopes.items():
        for name in names:
            entity = get_entity(data, cat, name)
            if entity:
                old = entity.get('scopes', {}).get('usage', [])
                if set(old) != set(ALL_SCOPES):
                    set_scopes(entity, ALL_SCOPES)
                    changes.append(f"{cat}/{name}: {old} -> {ALL_SCOPES}")
    
    # ==============================================================
    # CATEGORY 2: Country scopes (not found entities with correct scopes)
    # These are already correctly set to Country - keep as-is.
    # But some had overly broad scopes that need narrowing.
    # ==============================================================
    
    # ==============================================================
    # CATEGORY 3: Parenthesized triggers - verify/fix scopes
    # ==============================================================
    paren_fixes = {
        'triggers': {
            '(building_count_trigger)': ['Country', 'State'],  # wiki: can be used in building OR country scope
            '(ideology_support_trigger)': ['Country'],  # ideology popularity is Country-scoped
            '(resource_count_trigger)': ['Country', 'State'],  # wiki: can be used in country or state scope
        }
    }
    for cat, fixes in paren_fixes.items():
        for name, scopes in fixes.items():
            entity = get_entity(data, cat, name)
            if entity:
                old = entity.get('scopes', {}).get('usage', [])
                if set(old) != set(scopes):
                    set_scopes(entity, scopes)
                    changes.append(f"{cat}/{name}: {old} -> {scopes}")
    
    # ==============================================================
    # CATEGORY 4: "not found in vanilla" - triggers with current scopes
    # to verify and potentially fix
    # ==============================================================
    not_found_fixes = {
        'triggers': {
            # These have overly broad scopes that should be narrowed
            # threat -> Country (it's a country-level mechanic: world tension)
            'threat': ['Country'],
            # core_compliance -> Country (compliance is country-level for cores)
            'core_compliance': ['Country'],
            # core_resistance -> Country
            'core_resistance': ['Country'],
            # garrison_manpower_need -> Country (garrison need is calculated per country)
            'garrison_manpower_need': ['Country'],
            # has_country_leader -> Country (checking if country has a leader)
            'has_country_leader': ['Country'],
            # num_units -> Country (counting units)
            'num_units': ['Country'],
            # state -> already set correctly
            # Some triggers that are Character-scoped:
            'attack_skill_level': ['Character'],
            'defense_skill_level': ['Character'],
            'planning_skill_level': ['Character'],
            'logistics_skill_level': ['Character'],
            'skill_advantage': ['Character'],
            'can_select_trait': ['Character'],
            'dig_in': ['Unit', 'Character'],  # dig_in is a unit stat but also checked on leaders
            'fastest_unit': ['Character'],  # unit leader trait check
            'has_artillery_ratio': ['Character'],  # leader trait trigger
            'is_high_command': ['Character'],
            'is_navy_chief': ['Character'],
            'is_theorist': ['Character'],
            'is_scientist_injured': ['Character'],
            'has_scientist_level': ['Character'],
            'has_scientist_specialization': ['Character'],
            'has_mastery': ['Character'],
            'has_mastery_level': ['Character'],
            'has_mio_flag': ['Character', 'Country'],  # MIO flags can be on country or character
            'has_mio_number_of_completed_traits': ['Character', 'Country'],
            'has_mio_policy': ['Character', 'Country'],
            'has_mio_policy_active': ['Character', 'Country'],
            'has_mio_size': ['Character', 'Country'],
            'is_mio_assigned_to_task': ['Character'],
            'is_mio_trait_available': ['Character'],
            'is_mio_visible': ['Character'],
            'is_unit_leader': ['Character'],
            'is_political_advisor': ['Character'],
            'is_hired_as_advisor': ['Character'],
            'is_country_leader': ['Character'],
            # Unit-scoped triggers:
            'unit_organization': ['Unit'],
            'unit_strength': ['Unit'],
            'average_stats': ['Unit'],
            'min_planning': ['Unit'],
            'reserves': ['Unit'],
            'less_combat_width_than_opponent': ['Unit'],
            'is_fighting_air_units': ['Unit'],
            'is_leading_army_in_province': ['Unit', 'Character'],
            'is_unit_template_reserves': ['Unit'],
            'has_max_planning': ['Unit'],
            # State-scoped triggers:
            'compliance': ['State'],
            'compliance_speed': ['State'],
            'resistance': ['State'],
            'resistance_speed': ['State'],
            'free_building_slots': ['State'],
            'has_contested_owner': ['State'],
            'days_since_last_strategic_bombing': ['State'],
            'distance_to': ['State'],
            'state_population': ['State'],
            'state_population_k': ['State'],
            'state_strategic_value': ['State'],
            'state_and_terrain_strategic_value': ['State'],
            'temperature': ['State'],
            'has_railway_connection': ['State'],
            'has_railway_level': ['State'],
            'any_province_building_level': ['State'],
            'non_damaged_building_level': ['State'],
            # Country-scoped - economics/military:
            'has_army_size': ['Country'],
            'has_navy_size': ['Country'],
            'has_deployed_air_force_size': ['Country'],
            'has_equipment': ['Country'],
            'has_fuel': ['Country'],
            'has_manpower': ['Country'],
            'has_army_manpower': ['Country'],
            'has_political_power': ['Country'],
            'has_stability': ['Country'],
            'has_war_support': ['Country'],
            'has_legitimacy': ['Country'],
            'has_air_experience': ['Country'],
            'has_army_experience': ['Country'],
            'has_navy_experience': ['Country'],
            'has_any_grand_doctrine': ['Country'],
            'has_any_license': ['Country'],
            'has_license': ['Country'],
            'has_built': ['Country'],
            'has_active_rule': ['Country'],
            'has_game_rule': ['Country'],
            'has_allowed_idea_with_traits': ['Country'],
            'has_available_idea_with_traits': ['Country'],
            'has_track': ['Country'],  # Note: this might be misspelled 'has_completed_track' is in ambiguous
            'has_opinion': ['Country'],
            'has_relation_modifier': ['Country'],
            'has_power_balance': ['Country'],
            'has_power_balance_modifier': ['Country'],
            'has_added_tension_amount': ['Country'],
            'has_bombing_war_support': ['Country'],
            'has_casualties_war_support': ['Country'],
            'has_convoys_war_support': ['Country'],
            'has_attache': ['Country'],
            'has_collaboration': ['Country'],
            'has_country_custom_difficulty_setting': ['Country'],
            'has_done_agency_upgrade': ['Country'],
            'has_elections': ['Country'],
            'has_enemy_naval_control': ['Country'],
            'has_faction_goal': ['Country'],
            'has_finished_collecting_for_operation': ['Country'],
            'has_manpower_for_recruit_change_to': ['Country'],
            'has_military_industrial_organization': ['Country'],
            'has_mined': ['Country', 'State'],  # mining can be country or state
            'has_mines': ['Country', 'State'],
            'has_occupation_modifier': ['Country', 'State'],
            'has_officer_name': ['Country'],
            'has_operation_token': ['Country'],
            'has_resources_amount': ['Country'],
            'has_resources_in_country': ['Country'],
            'has_resources_rights': ['Country'],
            'has_tech_bonus': ['Country'],
            'has_volunteers_amount_from': ['Country'],
            'has_war_with_wargoal_against': ['Country'],
            'has_border_war_between': ['Country'],
            # Economic/military values:
            'agency_upgrade_number': ['Country'],
            'ai_irrationality': ['Country'],
            'ai_liberate_desire': ['Country'],
            'ai_wants_divisions': ['Country'],
            'alliance_naval_strength_ratio': ['Country'],
            'alliance_strength_ratio': ['Country'],
            'amount_manpower_in_deployment_queue': ['Country'],
            'amount_research_slots': ['Country'],
            'amount_taken_ideas': ['Country'],
            'any_claim': ['Country'],
            'any_war_score': ['Country'],
            'army_manpower_in_state': ['Country', 'State'],
            'buyer': ['Country'],
            'can_build_railway': ['Country', 'State'],
            'casualties': ['Country'],
            'casualties_inflicted_by': ['Country'],
            'casualties_k': ['Country'],
            'civilwar_target': ['Country'],
            'command_power_daily': ['Country'],
            'compare_autonomy_progress_ratio': ['Country'],
            'compare_autonomy_state': ['Country'],
            'compare_ideology_with_faction': ['Country'],
            'compare_intel_with': ['Country'],
            'conscription_ratio': ['Country'],
            'contract_contains_equipment': ['Country'],
            'convoy_threat': ['Country'],
            'current_conscription_amount': ['Country'],
            'days_since_capitulated': ['Country'],
            'deal_completion': ['Country'],
            'decryption_progress': ['Country'],
            'divisions_in_border_state': ['Country'],
            'divisions_in_state': ['Country'],
            'enemies_naval_strength_ratio': ['Country'],
            'enemies_strength_ratio': ['Country'],
            'estimated_intel_max_armor': ['Country'],
            'estimated_intel_max_piercing': ['Country'],
            'faction_goal_fulfillment': ['Country'],
            'faction_influence_rank': ['Country'],
            'faction_influence_ratio': ['Country'],
            'faction_influence_score': ['Country'],
            'faction_manifest_fulfillment': ['Country'],
            'faction_power_projection': ['Country'],
            'faction_upgrade_level': ['Country'],
            'fighting_army_strength_ratio': ['Country'],
            'focus_progress': ['Country'],
            'foreign_manpower': ['Country'],
            'fuel_ratio': ['Country'],
            'ic_ratio': ['Country'],
            'intel_level_over': ['Country'],
            'is_active_decryption_bonuses_enabled': ['Country'],
            'is_cryptology_department_active': ['Country'],
            'is_defender': ['Country'],
            'is_free_or_subject_of_root': ['Country'],
            'is_in_peace_conference': ['Country'],
            'is_licensing_to': ['Country'],
            'is_owner_neighbor_of': ['Country'],
            'is_power_balance_in_range': ['Country'],
            'is_power_balance_side_active': ['Country'],
            'is_preparing_operation': ['Country'],
            'is_running_operation': ['Country'],
            'is_staging_coup': ['Country'],
            'is_target_of_coup': ['Country'],
            'land_doctrine_level': ['Country'],
            'longest_war_length': ['Country'],
            'manpower_per_military_factory': ['Country'],
            'mine_threat': ['State', 'StrategicRegion'],
            'naval_strength_comparison': ['Country'],
            'naval_strength_ratio': ['Country'],
            'network_national_coverage': ['Country'],
            'night': ['Global'],  # night is a global condition, not scoped
            'num_battalions_in_states': ['Country'],
            'num_divisions': ['Country'],
            'num_divisions_in_states': ['Country'],
            'num_faction_members': ['Country'],
            'num_fake_intel_divisions': ['Country'],
            'num_finished_operations': ['Country'],
            'num_free_operative_slots': ['Country'],
            'num_occupied_states': ['Country'],
            'num_of_available_civilian_factories': ['Country'],
            'num_of_available_military_factories': ['Country'],
            'num_of_available_naval_factories': ['Country'],
            'num_of_civilian_factories': ['Country'],
            'num_of_civilian_factories_available_for_projects': ['Country'],
            'num_of_controlled_factories': ['Country'],
            'num_of_controlled_states': ['Country'],
            'num_of_factories': ['Country'],
            'num_of_military_factories': ['Country'],
            'num_of_naval_factories': ['Country'],
            'num_of_nukes': ['Country'],
            'num_of_operatives': ['Country'],
            'num_of_owned_factories': ['Country'],
            'num_of_supply_nodes': ['Country'],
            'num_operative_slots': ['Country'],
            'num_owned_neighbour_states': ['Country'],
            'num_planes_stationed_in_regions': ['Country'],
            'num_researched_technologies': ['Country'],
            'num_subjects': ['Country'],
            'num_tech_sharing_groups': ['Country'],
            'original_research_slots': ['Country'],
            'owns_any_state_of': ['Country'],
            'pc_current_score': ['Country'],
            'pc_does_state_stack_dismantled': ['Country'],
            'pc_is_liberated_by': ['Country'],
            'pc_is_on_winning_side': ['Country'],
            'pc_is_state_claimed': ['Country'],
            'pc_is_state_claimed_and_taken_by': ['Country'],
            'pc_is_untouched_loser': ['Country'],
            'pc_total_score': ['Country'],
            'pc_turn': ['Country'],
            'political_power_daily': ['Country'],
            'political_power_growth': ['Country'],
            'power_balance_daily_change': ['Country'],
            'power_balance_value': ['Country'],
            'power_balance_weekly_change': ['Country'],
            'received_expeditionary_forces': ['Country'],
            'recon_advantage': ['Country', 'Unit'],
            'ships_in_area': ['Country'],
            'ships_in_state_ports': ['Country', 'State'],
            'state': ['Country'],  # 'state' as trigger is different from State scope
            'strength_ratio': ['Country'],
            'surrender_progress': ['Country'],
            'target_conscription_amount': ['Country'],
            'war_length_with': ['Country'],
            # any_ / all_ scopes - these are Country scopes
            'any_country': ['Country'],
            'any_occupied_country': ['Country'],
            'all_country_with_original_tag': ['Country'],
            'any_other_country': ['Country'],
            'all_character': ['Character'],
            'any_country_division': ['Country', 'Unit'],
            'all_guaranteed_country': ['Country'],
            'any_army_leader': ['Character'],
            'any_state_of': ['State'],
            'any_guaranteed_country': ['Country'],
            'any_state_in': ['State'],
            'any_active_scientist': ['Character'],
            'any_neighbor_state': ['State'],
            'any_country_with_original_tag': ['Country'],
            'any_navy_leader': ['Character'],
            'all_subject_countries': ['Country'],
            'all_occupied_country': ['Country'],
            'all_neighbor_state': ['State'],
            'any_operative_leader': ['Character'],
            'all_scientists': ['Character'],
            'any_enemy_country': ['Country'],
            'all_army_leader': ['Character'],
            'all_operative_leader': ['Character'],
            'all_controlled_state': ['State'],
            'all_purchase_contract': ['Country'],
            'any_home_area_neighbor_country': ['Country'],
            'any_unit_leader': ['Character'],
            'all_neighbor_country': ['Country'],
            'all_state': ['State'],
            'any_controlled_state': ['State'],
            'all_other_country': ['Country'],
            'any_scientist': ['Character'],
            'any_country_with_core': ['Country'],
            'any_character': ['Character'],
            'all_allied_country': ['Country'],
            'any_core_state': ['State'],
            'any_neighbor_country': ['Country'],
            'all_active_scientist': ['Character'],
            'all_core_state': ['State'],
            'all_country': ['Country'],
            'all_unit_leader': ['Character'],
            'any_country_of': ['Country'],
            'any_allied_country': ['Country'],
            'any_state': ['State'],
            'any_state_division': ['State', 'Unit'],
            'all_navy_leader': ['Character'],
            'any_military_industrial_organization': ['Country', 'Character'],
            'any_owned_state': ['State'],
            'any_purchase_contract': ['Country'],
            'all_enemy_country': ['Country'],
            'all_country_of': ['Country'],
            'any_subject_country': ['Country'],
            'all_owned_state': ['State'],
            'all_military_industrial_organization': ['Country', 'Character'],
        },
        'effects': {
            # Scope referentials
            'capital_scope': ALL_SCOPES,
            'overlord': ALL_SCOPES,
            
            # Meta/utility effects
            'add_ability': ['Character'],
            'add_ace': ['Country'],
            'add_advisor_role': ['Country'],
            'add_ai_strategy': ['Country'],
            'add_autonomy_ratio': ['Country'],
            'add_autonomy_score': ['Country'],
            'add_breakthrough_points': ['Country'],
            'add_breakthrough_progress': ['Country'],
            'add_building_construction': ['State'],
            'add_collaboration': ['Country'],
            'add_corps_commander_role': ['Character'],
            'add_country_leader_role': ['Character'],
            'add_daily_mastery': ['Character'],
            'add_days_mission_timeout': ['Unit'],
            'add_days_remove': ['Character'],
            'add_decryption': ['Country'],
            'add_design_template_bonus': ['Country'],
            'add_doctrine_cost_reduction': ['Country'],
            'add_dynamic_modifier': ALL_SCOPES,
            'add_equipment_bonus': ['Country'],
            'add_equipment_production': ['Country'],
            'add_equipment_subsidy': ['Country'],
            'add_equipment_to_stockpile': ['Country'],
            'add_faction_goal_slot': ['Country'],
            'add_faction_influence_score': ['Country'],
            'add_field_marshal_role': ['Character'],
            'add_history_entry': ALL_SCOPES,
            'add_intel': ['Country'],
            'add_mastery': ['Character'],
            'add_mastery_bonus': ['Character'],
            'add_max_trait': ['Character'],
            'add_mines': ['State'],
            'add_mio_design_team_change_cost': ['Country'],
            'add_mio_industrial_manufacturer_assign_cost': ['Country'],
            'add_mio_policy_cooldown': ['Country'],
            'add_mio_policy_cost': ['Country'],
            'add_mio_size_up_requirement_factor': ['Country'],
            'add_mio_task_capacity': ['Country'],
            'add_named_threat': ['Country'],
            'add_naval_commander_role': ['Character'],
            'add_offsite_building': ['Country'],
            'add_operation_token': ['Country'],
            'add_opinion_modifier': ['Country'],
            'add_popularity': ['Country'],
            'add_power_balance_modifier': ['Country'],
            'add_power_balance_value': ['Country'],
            'add_province_modifier': ['State'],
            'add_random_trait': ['Character'],
            'add_random_valid_trait_from_unit': ['Character'],
            'add_relation_modifier': ['Country'],
            'add_relation_rule_override': ['Country'],
            'add_resource': ['Country', 'State'],
            'add_scientist_level': ['Character'],
            'add_scientist_role': ['Character'],
            'add_scientist_xp': ['Character'],
            'add_state_modifier': ['State'],
            'add_state_resistance_compliance_modifier': ['State'],
            'add_tech_bonus': ['Country'],
            'add_temporary_buff_to_units': ['Country'],
            'add_timed_idea': ['Country'],
            'add_timed_unit_leader_trait': ['Character'],
            'add_to_array': ALL_SCOPES,
            'add_to_variable': ALL_SCOPES,
            'add_to_war': ['Country'],
            'add_trait': ['Character'],
            'add_unit_bonus': ['Unit', 'Country'],
            'add_unit_medal_to_latest_entry': ['Character'],
            'add_units_to_division_template': ['Country'],
            'add_victory_points': ['State'],
            'annex_country': ['Country'],
            'become_exiled_in': ['Country'],
            'build_railway': ['State'],
            'cancel_border_war': ['Country'],
            'cancel_purchase_contract': ['Country'],
            'capture_operative': ['Country'],
            'career_profile_set_temp_playthrough_variable': ['Global'],
            'career_profile_set_temp_variable': ['Global'],
            'change_division_template': ['Unit', 'Country'],
            'clamp_variable': ALL_SCOPES,
            'clear_division_template_cap': ['Country'],
            'clear_global_event_targets': ['Global'],
            'clr_mio_flag': ['Country', 'Character'],
            'clr_project_flag': ['Country'],
            'construct_building_in_random_province': ['State'],
            'create_colonial_division_template': ['Country'],
            'create_corps_commander': ['Character'],
            'create_country_leader': ['Character'],
            'create_dynamic_country': ['Country'],
            'create_entity': ['Country'],  # Special projects
            'create_equipment_variant': ['Country'],
            'create_field_marshal': ['Character'],
            'create_import': ['Country'],
            'create_navy_leader': ['Character'],
            'create_operative_leader': ['Character'],
            'create_production_license': ['Country'],
            'create_purchase_contract': ['Country'],
            'create_railway_gun': ['Country'],
            'create_ship': ['Country'],
            'create_unit': ['Country'],
            'create_wargoal': ['Country'],
            'custom_effect_tooltip': ALL_SCOPES,
            'damage_building': ['State'],
            'damage_units': ['State', 'Country'],
            'declare_war_on': ['Country'],
            'delete_unit': ['Country', 'Unit'],
            'delete_unit_template_and_units': ['Country'],
            'delete_units': ['Country', 'Unit'],
            'destroy_entity': ['Country'],
            'destroy_resource': ['State'],
            'destroy_ships': ['Country', 'Unit'],
            'destroy_unit': ['Country', 'Unit'],
            'diplomatic_relation': ['Country'],
            'divide_variable': ALL_SCOPES,
            'effect_tooltip': ALL_SCOPES,
            # every_ scopes
            'every_active_scientist': ['Character'],
            'every_allied_country': ['Country'],
            'every_army_leader': ['Character'],
            'every_character': ['Character'],
            'every_collection_element': ALL_SCOPES,
            'every_controlled_state': ['State'],
            'every_core_state': ['State'],
            'every_country': ['Country'],
            'every_country_division': ['Country', 'Unit'],
            'every_country_with_original_tag': ['Country'],
            'every_enemy_country': ['Country'],
            'every_faction_member': ['Country'],
            'every_military_industrial_organization': ['Country', 'Character'],
            'every_navy_leader': ['Character'],
            'every_neighbor_country': ['Country'],
            'every_neighbor_state': ['State'],
            'every_occupied_country': ['Country'],
            'every_operative': ['Character'],
            'every_other_country': ['Country'],
            'every_owned_state': ['State'],
            'every_possible_country': ['Country'],
            'every_purchase_contract': ['Country'],
            'every_scientist': ['Character'],
            'every_state': ['State'],
            'every_state_division': ['State', 'Unit'],
            'every_subject_country': ['Country'],
            'every_unit_leader': ['Character'],
            'execute_operation_coordinated_strike': ['Country'],
            'finalize_border_war': ['Country'],
            'find_highest_in_array': ALL_SCOPES,
            'find_lowest_in_array': ALL_SCOPES,
            'force_update_map_mode': ['Global'],
            'free_operative': ['Character'],
            'free_random_operative': ['Character', 'Country'],
            'generate_character': ['Character'],
            'generate_scientist_character': ['Character'],
            'get_highest_scored_country_temp': ALL_SCOPES,
            'get_sorted_scored_countries_temp': ALL_SCOPES,
            'get_supply_vehicles': ['Country'],
            'get_supply_vehicles_temp': ['Country'],
            'give_resource_rights': ['Country'],
            'global_every_army_leader': ['Global', 'Country', 'Character'],
            'goto_province': ['Global'],
            'kill_operative': ['Character'],
            'launch_nuke': ['State', 'Country'],
            'mark_technology_tree_layout_dirty': ['Global'],
            'modify_building_resources': ['State'],
            'modify_character_flag': ['Character'],
            'modify_country_flag': ['Country'],
            'modify_global_flag': ['Global'],
            'modify_mio_flag': ['Country', 'Character'],
            'modify_project_flag': ['Country'],
            'modify_state_flag': ['State'],
            'modify_tech_sharing_bonus': ['Country'],
            'modify_timed_idea': ['Country'],
            'modify_unit_leader_flag': ['Character'],
            'modulo_variable': ALL_SCOPES,
            'multiply_variable': ALL_SCOPES,
            'operative_leader_event': ['Character'],
            'party_leader': ['Country'],
            'promote_officer_to_general': ['Character'],
            'raid_damage_units': ['Country'],
            # random_ scopes
            'random_active_scientist': ['Character'],
            'random_allied_country': ['Country'],
            'random_army_leader': ['Character'],
            'random_character': ['Character'],
            'random_controlled_state': ['State'],
            'random_core_state': ['State'],
            'random_country': ['Country'],
            'random_country_division': ['Country', 'Unit'],
            'random_country_with_original_tag': ['Country'],
            'random_enemy_country': ['Country'],
            'random_military_industrial_organization': ['Country', 'Character'],
            'random_navy_leader': ['Character'],
            'random_neighbor_country': ['Country'],
            'random_neighbor_state': ['State'],
            'random_occupied_country': ['Country'],
            'random_operative': ['Character'],
            'random_other_country': ['Country'],
            'random_owned_controlled_state': ['State'],
            'random_owned_state': ['State'],
            'random_purchase_contract': ['Country'],
            'random_scientist': ['Character'],
            'random_state': ['State'],
            'random_state_division': ['State', 'Unit'],
            'random_subject_country': ['Country'],
            'random_unit_leader': ['Character'],
            'reduce_focus_completion_cost': ['Country'],
            'release_autonomy': ['Country'],
            'remove_ability': ['Character'],
            'remove_advisor_role': ['Character'],
            'remove_all_power_balance_modifiers': ['Country'],
            'remove_building': ['State'],
            'remove_civil_war_target': ['Country'],
            'remove_contested_owner': ['State'],
            'remove_country_leader_role': ['Character'],
            'remove_dynamic_modifier': ALL_SCOPES,
            'remove_faction_goal': ['Country'],
            'remove_from_array': ALL_SCOPES,
            'remove_operation_token': ['Country'],
            'remove_opinion_modifier': ['Country'],
            'remove_power_balance': ['Country'],
            'remove_power_balance_modifier': ['Country'],
            'remove_province_modifier': ['State'],
            'remove_relation_modifier': ['Country'],
            'remove_relation_rule_override': ['Country'],
            'remove_resistance_target': ['Country'],
            'remove_state_resistance_compliance_modifier': ['State'],
            'remove_targeted_decision': ['Country'],
            'remove_trait': ['Character'],
            'remove_unit_leader': ['Character'],
            'remove_wargoal': ['Country'],
            'replace_unit_leader_trait': ['Character'],
            'reseed_division_commander': ['Character', 'Unit'],
            'resize_array': ALL_SCOPES,
            'reverse_add_opinion_modifier': ['Country'],
            'scoped_sound_effect': ALL_SCOPES,
            'send_equipment': ['Country'],
            'send_equipment_fraction': ['Country'],
            'set_autonomy': ['Country'],
            'set_border_war_data': ['Country'],
            'set_building_level': ['State'],
            'set_can_be_fired_in_advisor_role': ['Character'],
            'set_capital': ['State'],
            'set_collaboration': ['Country', 'State'],
            'set_country_leader_description': ['Character'],
            'set_country_leader_name': ['Character'],
            'set_country_leader_portrait': ['Character'],
            'set_division_force_allow_recruiting': ['Country', 'Unit'],
            'set_division_template_cap': ['Country'],
            'set_division_template_lock': ['Country'],
            'set_entity_animation': ['Country'],
            'set_entity_movement': ['Country'],
            'set_entity_position': ['Country'],
            'set_entity_rotation': ['Country'],
            'set_entity_scale': ['Country'],
            'set_equipment_version_number': ['Country'],
            'set_faction_member_upgrade_min': ['Country'],
            'set_faction_spymaster': ['Country'],
            'set_faction_upgrade': ['Country'],
            'set_fuel': ['Country'],
            'set_keyed_oob': ['Country'],
            'set_leader_name': ['Character'],
            'set_legitimacy': ['Country'],
            'set_mio_design_team_assign_cost': ['Country'],
            'set_mio_design_team_change_cost': ['Country'],
            'set_mio_flag': ['Country', 'Character'],
            'set_mio_funds_gain_factor': ['Country'],
            'set_mio_funds': ['Country'],
            'set_mio_industrial_manufacturer_assign_cost': ['Country'],
            'set_mio_policy_cooldown': ['Country'],
            'set_mio_policy_cost': ['Country'],
            'set_mio_research_bonus': ['Country'],
            'set_mio_size_up_requirement_factor': ['Country'],
            'set_mio_task_capacity': ['Country'],
            'set_party_name': ['Country'],
            'set_party_rule': ['Country'],
            'set_political_party': ['Country'],
            'set_political_power': ['Country'],
            'set_politics': ['Country'],
            'set_popularities': ['Country'],
            'set_portraits': ['Character'],
            'set_power_balance': ['Country'],
            'set_power_balance_gfx': ['Country'],
            'set_province_name': ['State'],
            'set_relation_rule': ['Country'],
            'set_rule': ['Country'],
            'set_state_owner': ['State'],
            'set_state_province_controller': ['State'],
            'set_technology': ['Country'],
            'set_temp_variable': ALL_SCOPES,
            'set_truce': ['Country'],
            'set_variable': ALL_SCOPES,
            'set_variable_to_random': ALL_SCOPES,
            'set_victory_points': ['State'],
            'start_border_war': ['Country'],
            'start_civil_war': ['Country'],
            'start_peace_conference': ['Country'],
            'state_event': ['State'],
            'steal_random_tech_bonus': ['Country'],
            'strategic_province_location': ['State'],
            'strategic_state_location': ['State'],
            'subtract_from_variable': ALL_SCOPES,
            'swap_country_leader_traits': ['Character'],
            'swap_ideas': ['Country'],
            'swap_ruler_traits': ['Character'],
            'teleport_armies': ['Country'],
            'transfer_ship': ['Country'],
            'transfer_units_fraction': ['Country'],
            'turn_operative': ['Character'],
            'uncomplete_national_focus': ['Country'],
            'unlock_mio_policy_tooltip': ['Country'],
            'transfer_navy': ['Country'],
            'activate_targeted_decision': ['Country'],
            'add_cic': ['Country'],
            'add_claim_by': ['State'],
            'add_compliance': ['State'],
            'add_core_of': ['State'],
            'add_faction_influence_ratio': ['Country'],
            'add_manpower': ['Country', 'State'],  # can add manpower to country or state
            'add_political_power': ['Country'],
            'add_resistance': ['State'],
            'add_resistance_target': ['Country'],
            'add_skill_level': ['Character'],
            'add_stability': ['Country'],
            'add_state_claim': ['Country'],
            'add_war_support': ['Country'],
            'clear_array': ALL_SCOPES,
            'clr_character_flag': ['Character'],
            'clr_country_flag': ['Country'],
            'country_event': ['Country'],
            'drop_cosmetic_tag': ['Country'],
            'gain_xp': ['Character'],
            'instantiate_collaboration_government': ['Country'],
            'load_oob': ['Country'],
            'promote_character': ['Country', 'Character'],
            'remove_claim_by': ['State'],
            'remove_core_of': ['State'],
            'remove_country_leader_trait': ['Character', 'Country'],
            'remove_decision_on_cooldown': ['Country'],
            'remove_ideas': ['Country'],
            'remove_resource_rights': ['Country'],
            'retire_character': ['Country', 'Character'],
            'set_character_flag': ['Character'],
            'set_character_name': ['Character'],
            'set_cosmetic_tag': ['Country'],
            'set_country_flag': ['Country'],
            'set_global_flag': ALL_SCOPES,
            'set_major': ['Country'],
            'set_nationality': ['Character'],
            'set_state_category': ['State'],
            'set_state_name': ['State'],
            'start_resistance': ['State'],
            'transfer_state_to': ['Country', 'State'],
            'unlock_national_focus': ['Country'],
            'activate_decision': ['Country'],
        },
        'modifiers': {
            # Many modifiers already have correct Country scope.
            # These are the ones that need broader scopes or different scopes.
            'operative_slot': ['Country'],
            'naval_retreat_chance_after_initial_combat': ['Country', 'Global', 'Character', 'Idea'],
            'naval_attrition': ['Country', 'Global'],
            'naval_strike_targetting_factor': ['Country', 'Character', 'Global', 'Ace'],
            'global_building_slots': ['Country', 'State', 'Global', 'Character'],
            'local_resources_factor': ['Country', 'State', 'Global'],
            'experience_loss_factor': ['Country', 'Character', 'Global'],
            'army_morale_factor': ['Country', 'Character', 'Global'],
            'naval_speed_factor': ['Country', 'Character', 'Unit', 'Global'],
            'intelligence_agency_defense': ['Country', 'Character', 'Global', 'Idea'],
            'refit_speed': ['Country', 'Global', 'Idea'],
            'pocket_penalty': ['Country', 'Character', 'Global'],
            'navy_screen_attack_factor': ['Country', 'Character', 'Global', 'Idea'],
            'shore_bombardment_bonus': ['Character', 'Country', 'Global'],
            'recon_factor': ['Country', 'Character', 'Global', 'Idea'],
            'navy_submarine_attack_factor': ['Country', 'Character', 'Global', 'Idea'],
            'weekly_bombing_war_support': ['Country', 'Character', 'Global', 'Idea'],
            'paratrooper_weight_factor': ['Character', 'Country', 'Global'],
            'amphibious_invasion': ['Country', 'Character', 'Global'],
            'command_power_gain_mult': ['Country', 'Character', 'Global'],
            'enemy_army_bonus_air_superiority_factor': ['Country', 'Character', 'Global'],
            'production_factory_efficiency_gain_factor': ['Country', 'Character', 'Global', 'Idea'],
            'land_bunker_effectiveness_factor': ['Country', 'Character', 'Global'],
            'navy_submarine_detection_factor': ['Country', 'Character', 'Global'],
            'industry_free_repair_factor': ['Country', 'Character', 'Global', 'Idea'],
            'local_factory_energy_consumption': ['State', 'Global'],
            'ai_call_ally_desire_factor': ['Country', 'Character', 'Global'],
            'offensive_war_stability_factor': ['Country', 'Character', 'Global', 'Idea'],
            'air_detection': ['StrategicRegion', 'Global', 'Country'],
            'generate_wargoal_tension': ['Country', 'Character', 'Global'],
            'army_speed_factor': ['Country', 'Character', 'Global'],
            'weekly_casualties_war_support': ['Country', 'Character', 'Global'],
            'river_crossing_factor': ['Country', 'Character', 'Global'],
            'experience_gain_air': ['Country', 'Character', 'Global'],
            'naval_invasion_prep_speed': ['Character', 'Country', 'Global'],
            'navy_anti_air_attack_factor': ['Country', 'Character', 'Global'],
            'army_strength_factor': ['Country', 'Character', 'Global', 'Idea'],
            'navy_carrier_air_targetting_factor': ['Country', 'Character', 'Global'],
            'resistance_target_on_our_occupied_states': ['Country', 'Character', 'Global'],
            'resistance_decay_on_our_occupied_states': ['Country', 'Character', 'Global'],
            'fighter_sortie_efficiency': ['Country', 'Character', 'Global', 'Idea'],
            'carrier_night_traffic': ['Country', 'Global', 'Idea'],
            'refit_ic_cost': ['Country', 'Global', 'Idea'],
            'out_of_supply_factor': ['Country', 'Character', 'Global'],
            'promote_cost_factor': ['Country', 'Character', 'Global', 'Idea'],
            'root_out_resistance_effectiveness_factor': ['Country', 'Character', 'Global'],
            'resistance_target': ['Country', 'Character', 'Global'],
            'army_defence_against_major_factor': ['Country', 'Character', 'Global'],
            'navy_org': ['Country', 'Character', 'Global'],
            'surrender_limit': ['Country', 'Character', 'Global', 'Idea'],
            'air_fuel_consumption_factor': ['Country', 'Character', 'Global', 'Idea'],
            'no_supply_grace': ['Character', 'Country', 'Global'],
            'justify_war_goal_time': ['Country', 'Character', 'Global'],
            'experience_gain_army_unit_factor': ['Country', 'Character', 'Global', 'Idea'],
            'factor': ['Country', 'Global', 'Character', 'Idea', 'State', 'Unit'],
            'naval_defense_factor': ['Country', 'Character', 'Global'],
            'non_core_manpower': ['Country', 'State', 'Character', 'Global'],
            'experience_gain_army_factor': ['Country', 'Character', 'Global'],
            'fuel_gain_factor_from_states': ['Country', 'Character', 'Global', 'Idea'],
            'land_night_attack': ['Country', 'Character', 'Global'],
            'army_attack_factor': ['Country', 'Character', 'Global'],
            'air_untrained_pilots_penalty_factor': ['Country', 'Global', 'Idea'],
            'production_factory_max_efficiency_factor': ['Country', 'Character', 'Global', 'Idea'],
            'supply_combat_penalties_on_core_factor': ['Country', 'Character', 'Global'],
            'air_superiority_bonus_in_combat': ['Country', 'Character', 'Global'],
            'justify_war_goal_when_in_major_war_time': ['Country', 'Character', 'Global'],
            'drift_defence_factor': ['Country', 'Character', 'Global'],
            'cic_construction_boost_factor': ['Country', 'Global', 'Idea'],
            'max_dig_in': ['Country', 'Character', 'Global'],
            'faction_trade_opinion_factor': ['Country', 'Character', 'Global', 'Idea'],
            'naval_mines_damage_factor': ['Country', 'Global'],
            'field_officer_promotion_penalty': ['Country', 'Global', 'Idea'],
            'resistance_damage_to_garrison_on_our_occupied_states': ['Country', 'Character', 'Global'],
            'war_stability_factor': ['Country', 'Character', 'Global'],
            'initiative_factor': ['Country', 'Character', 'Global'],
            'experience_gain_navy': ['Country', 'Character', 'Global'],
            'resistance_growth': ['Country', 'State', 'Character', 'Global'],
            'army_attack_against_major_factor': ['Country', 'Character', 'Global'],
            'navy_capital_ship_attack_factor': ['Country', 'Character', 'Global'],
            'enemy_operative_capture_chance_factor': ['Country', 'Character', 'Global'],
            'experience_gain_army': ['Country', 'Character', 'Global'],
            'naval_torpedo_damage_reduction_factor': ['Country', 'Unit', 'Global'],
            'railway_gun_bombardment_factor': ['Country', 'Global', 'Idea'],
            'war_support_factor': ['Country', 'Character', 'Global', 'Idea'],
            'enemy_declare_war_tension': ['Country', 'Character', 'Global'],
            'naval_morale_factor': ['Country', 'Global'],
            'acclimatization_hot_climate_gain_factor': ['Country', 'Global'],
            'fortification_collateral_chance': ['Character', 'Global'],
            'max_surrender_limit_offset': ['Country', 'Character', 'Global', 'Idea'],
            'global_building_slots_factor': ['Country', 'Character', 'Global', 'Idea'],
            'resistance_damage_to_garrison': ['Country', 'State', 'Character', 'Global'],
            'max_planning': ['Country', 'Character', 'Global'],
            'guarantee_cost': ['Country', 'Character', 'Global'],
            'min_export': ['Country', 'Character', 'Global', 'Idea'],
            'air_accidents_factor': ['Country', 'Character', 'Global'],
            'mobilization_speed': ['Country', 'State', 'Character', 'Global'],
            'industry_air_damage_factor': ['Country', 'Global'],
            'naval_coordination': ['Country', 'Character', 'Global'],
            'navy_carrier_air_attack_factor': ['Country', 'Character', 'Global'],
            'naval_torpedo_hit_chance_factor': ['Country', 'Character', 'Global', 'Unit', 'Idea'],
            'send_volunteer_factor': ['Country', 'Character', 'Global'],
            'party_popularity_stability_factor': ['Country', 'Character', 'Global', 'Idea'],
            'acclimatization_cold_climate_gain_factor': ['Character', 'Country', 'Global'],
            'intel_from_combat_factor': ['Country', 'Character', 'Global', 'Idea'],
            'sortie_efficiency': ['Country', 'Character', 'Global', 'Idea'],
            'fuel_gain_factor': ['Country', 'Global', 'Idea'],
            'exile_manpower_factor': ['Country', 'Character', 'Global'],
            'weekly_convoys_war_support': ['Country', 'Character', 'Global', 'Idea'],
            'terrain_trait_xp_gain_factor': ['Country', 'Character', 'Global', 'Idea'],
            'operation_outcome': ['Country', 'Character', 'Global'],
            'equipment_conversion_speed': ['Country', 'Character', 'Global', 'Idea'],
            'navy_screen_defence_factor': ['Country', 'Character', 'Global'],
            'intel_network_gain': ['Country', 'Character', 'Global'],
            'production_factory_start_efficiency_factor': ['Country', 'Character', 'Global', 'Idea'],
            'resistance_garrison_penetration_chance': ['Country', 'State', 'Character', 'Global'],
            'experience_gain_factor': ['Country', 'Character', 'Global', 'Idea'],
            'resistance_decay': ['Country', 'State', 'Character', 'Global'],
            'planning_speed': ['Country', 'Character', 'State', 'Global'],
            'dig_in_speed_factor': ['Country', 'Character', 'Global'],
            'navy_carrier_air_agility_factor': ['Country', 'Character', 'Global', 'Idea'],
            'supply_consumption_factor': ['Country', 'Character', 'Global'],
            'resource_trade_cost_bonus_per_factory': ['Country', 'Character', 'Global', 'Idea'],
            'mines_sweeping_by_fleets_factor': ['Country', 'Character', 'Global'],
            'air_agility_factor': ['Country', 'Global', 'Idea', 'Ace'],
            'army_core_attack_factor': ['Country', 'Character', 'Global'],
            'equipment_capture_factor': ['Country', 'Character', 'Global', 'Unit', 'Idea'],
            'naval_retreat_speed': ['Country', 'Character', 'Global', 'Idea'],
            'naval_critical_effect_factor': ['Country', 'Character', 'Global', 'Idea'],
            'compliance_growth': ['Country', 'State', 'Character', 'Global'],
            'resistance_activity': ['Country', 'Character', 'Global'],
            'intel_network_gain_factor': ['Country', 'Character', 'Global'],
            'legitimacy_daily': ['Country', 'Character', 'Global'],
            'grant_medal_cost_factor': ['Country', 'Character', 'Global', 'Idea'],
            'army_defence_factor': ['Country', 'Character', 'Global'],
            'equipment_upgrade_xp_cost': ['Country', 'Character', 'Global'],
            'stability_factor': ['Country', 'Character', 'Global', 'Idea', 'State'],
            'research_speed_factor': ['Country', 'Character', 'Global', 'Idea'],
            'naval_light_gun_hit_chance_factor': ['Country', 'Character', 'Global', 'Unit'],
            'navy_org_factor': ['Country', 'Character', 'Global'],
            'breakthrough_bonus_against': ['Country', 'Global', 'Idea'],
            'commando_trait_chance_factor': ['Country', 'Character', 'Global'],
            'naval_night_attack': ['Country', 'Character', 'Global', 'Idea'],
            'positioning': ['Country', 'Character', 'Global'],
            'monthly_population': ['Country', 'Character', 'Global'],
            'trade_opinion_factor': ['Country', 'Character', 'Global'],
            'unit_leader_as_advisor_cp_cost_factor': ['Country', 'Global', 'Idea'],
            'political_power_factor': ['Country', 'Character', 'Global'],
            'truck_attrition_factor': ['Country', 'Global', 'Idea'],
            'resistance_growth_on_our_occupied_states': ['Country', 'Character', 'Global'],
            'naval_torpedo_screen_penetration_factor': ['Country', 'Character', 'Global', 'Idea'],
            'critical_receive_chance': ['Country', 'Character', 'Global', 'Idea'],
            'production_lack_of_resource_penalty_factor': ['Country', 'Character', 'Global', 'Idea'],
            'request_lease_tension': ['Country', 'Character', 'Global'],
            'navy_fuel_consumption_factor': ['Country', 'Character', 'Global', 'Idea'],
            'line_change_production_efficiency_factor': ['Country', 'Character', 'Global', 'Idea'],
            'land_reinforce_rate': ['Country', 'Character', 'Global'],
            'subversive_activites_upkeep': ['Country', 'Character', 'Global', 'Idea'],
            'license_purchase_cost': ['Country', 'Global', 'Idea'],
            'supply_node_range': ['Country', 'Character', 'Global'],
            'equipment_capture': ['Country', 'Character', 'Global'],
            'agency_upgrade_time': ['Country', 'Character', 'Global', 'Idea'],
            'coordination_bonus': ['Country', 'Character', 'Global', 'Idea'],
            'license_production_speed': ['Country', 'Global'],
            'defensive_war_stability_factor': ['Country', 'Character', 'Global', 'Idea'],
            'air_mission_efficiency': ['Country', 'Global', 'Idea'],
            'conversion_cost_civ_to_mil_factor': ['Country', 'Character', 'Global', 'Idea'],
            'subjects_autonomy_gain': ['Country', 'Character', 'Global'],
            'enemy_justify_war_goal_time': ['Country', 'Character', 'Global'],
            'naval_detection': ['Country', 'Global'],
            'spotting_chance': ['Country', 'Character', 'Global'],
            'repair_speed_factor': ['Country', 'Global', 'Idea', 'State'],
            'compliance_gain': ['Country', 'State', 'Character', 'Global'],
            'annex_cost_factor': ['Country', 'Character', 'Global'],
            'naval_heavy_gun_hit_chance_factor': ['Country', 'Character', 'Global', 'Unit'],
            'reassignment_duration_factor': ['Country', 'Character', 'Global', 'Idea'],
            'extra_paratrooper_supply_grace': ['Country', 'Character', 'Global', 'Idea'],
            'army_attack_speed_factor': ['Country', 'Character', 'Global'],
            'army_core_defence_factor': ['Country', 'Character', 'Global'],
            'defence': ['Country', 'Character', 'Global', 'Unit'],
            'air_carrier_night_penalty_reduction_factor': ['Country', 'Global', 'Idea'],
            'naval_torpedo_enemy_critical_chance_factor': ['Country', 'Global', 'Unit'],
            'industrial_factory_donations': ['Country', 'Character', 'Global'],
            'navy_capital_ship_defence_factor': ['Country', 'Character', 'Global'],
            'naval_accidents_chance': ['Country', 'Global', 'Idea'],
            'max_command_power_mult': ['Country', 'Character', 'Global'],
            'decryption_factor': ['Country', 'Character', 'Global'],
        },
    }
    
    # Apply corrections for each category
    for cat, fixes in not_found_fixes.items():
        for name, scopes in fixes.items():
            entity = get_entity(data, cat, name)
            if entity:
                old = entity.get('scopes', {}).get('usage', [])
                # Only update if different
                if set(old) != set(scopes):
                    set_scopes(entity, scopes)
                    changes.append(f"{cat}/{name}: {old} -> {scopes}")
    
    # ==============================================================
    # CATEGORY 5: Ambiguous scopes - entities that need careful narrowing
    # based on HOI4 domain knowledge
    # ==============================================================
    ambiguous_fixes = {
        'triggers': {
            # division_has_battalion_in_template -> Unit (template check is unit-level)
            'division_has_battalion_in_template': ['Unit'],
            # has_active_resistance -> State (resistance is state-level)
            'has_active_resistance': ['State'],
            # has_any_power_balance -> Country (power balance is national)
            'has_any_power_balance': ['Country'],
            # has_character -> Country (checking if a country has a character)
            'has_character': ['Country'],
            # has_completed_track -> Country (track is national)
            'has_completed_track': ['Country'],
            # has_country_flag -> Country (country flags are country-scoped)
            'has_country_flag': ['Country', 'Global', 'State', 'Character', 'Unit'],
            # Actually has_country_flag can be checked from any scope with a country 
            # but the trigger itself is Country-scoped if we're checking it on a country.
            # Vanilla says: Country, Global, Idea, Character, State
            'has_country_flag': ['Country', 'Global', 'State', 'Character', 'Unit', 'Idea'],
            # has_dlc -> Global (DLC check is global)
            'has_dlc': ALL_SCOPES,  # DLC check can be done from anywhere
            # has_done_agency_upgrade -> Country
            'has_done_agency_upgrade': ['Country'],
            # has_elections -> Country
            'has_elections': ['Country'],
            # has_event_target -> ALL scopes
            'has_event_target': ALL_SCOPES,
            # has_focus_tree -> Country
            'has_focus_tree': ['Country'],
            # has_idea -> Country, Character, Global, Idea, Unit
            'has_idea': ['Country', 'Character', 'Global', 'Idea', 'Unit'],
            # has_ideology_group -> Character, Country (characters and countries have ideologies)
            'has_ideology_group': ['Character', 'Country'],
            # has_military_access_to -> Country
            'has_military_access_to': ['Country'],
            # has_offensive_war -> Country
            'has_offensive_war': ['Country'],
            # has_resistance -> State
            'has_resistance': ['State'],
            # has_rule -> Country
            'has_rule': ['Country'],
            # has_template -> Country
            'has_template': ['Country'],
            # has_terrain -> Country (countries can check terrain in their states)
            'has_terrain': ['Country', 'State'],
            # has_war -> Country
            'has_war': ['Country'],
            # has_war_with_major -> Country
            'has_war_with_major': ['Country'],
            # impassable -> State
            'impassable': ['State'],
            # is_ai -> Country
            'is_ai': ['Country', 'Character'],
            # is_capital -> State
            'is_capital': ['State'],
            # is_claimed_by -> State
            'is_claimed_by': ['State', 'Country'],
            # is_coastal -> State
            'is_coastal': ['State'],
            # is_controlled_by -> State
            'is_controlled_by': ['State', 'Country'],
            # is_core_of -> State
            'is_core_of': ['State'],
            # is_country_leader -> Character
            'is_country_leader': ['Character'],
            # is_demilitarized_zone -> State
            'is_demilitarized_zone': ['State'],
            # is_dynamic_country -> Country
            'is_dynamic_country': ['Country'],
            # is_hired_as_advisor -> Character
            'is_hired_as_advisor': ['Character'],
            # is_historical_focus_on -> Global (it's a game setting)
            'is_historical_focus_on': ['Global', 'Country'],
            # is_in_faction_with -> Country
            'is_in_faction_with': ['Country'],
            # is_in_home_area -> State
            'is_in_home_area': ['State'],
            # is_island_state -> State
            'is_island_state': ['State'],
            # is_on_continent -> Country, State
            'is_on_continent': ['Country', 'State'],
            # is_owned_and_controlled_by -> State
            'is_owned_and_controlled_by': ['State'],
            # is_owned_by -> State
            'is_owned_by': ['State'],
            # is_political_advisor -> Character
            'is_political_advisor': ['Character'],
            # is_researching_technology -> Country
            'is_researching_technology': ['Country'],
            # is_unit_leader -> Character
            'is_unit_leader': ['Character'],
            # region -> State (strategic region scoping for states)
            'region': ['State'],
            # resistance_target -> Country
            'resistance_target': ['Country'],
            # tag -> Country (tag check is country-scoped)
            'tag': ['Country'],
            # threat -> Country (world tension is country)
            'threat': ['Country'],
            # owner -> depends but typically Country (owner of a scope)
            'owner': ALL_SCOPES,  # as a trigger scope keyword
        },
        'effects': {
            'add_cic': ['Country'],
            'add_claim_by': ['State'],
            'add_compliance': ['State'],
            'add_core_of': ['State'],
            'add_faction_influence_ratio': ['Country'],
            'add_manpower': ['Country', 'State'],
            'add_political_power': ['Country'],
            'add_resistance': ['State'],
            'add_resistance_target': ['Country'],
            'add_skill_level': ['Character'],
            'add_stability': ['Country'],
            'add_state_claim': ['Country'],
            'add_war_support': ['Country'],
            'clear_array': ALL_SCOPES,
            'clr_character_flag': ['Character'],
            'clr_country_flag': ['Country'],
            'country_event': ['Country'],
            'drop_cosmetic_tag': ['Country'],
            'gain_xp': ['Character'],
            'instantiate_collaboration_government': ['Country'],
            'load_oob': ['Country'],
            'owner': ALL_SCOPES,
            'promote_character': ['Country', 'Character'],
            'remove_claim_by': ['State'],
            'remove_core_of': ['State'],
            'remove_country_leader_trait': ['Character', 'Country'],
            'remove_decision_on_cooldown': ['Country'],
            'remove_ideas': ['Country'],
            'remove_resource_rights': ['Country'],
            'retire_character': ['Country', 'Character'],
            'save_event_target_as': ALL_SCOPES,
            'set_character_flag': ['Character'],
            'set_character_name': ['Character'],
            'set_cosmetic_tag': ['Country'],
            'set_country_flag': ['Country'],
            'set_global_flag': ALL_SCOPES,
            'set_major': ['Country'],
            'set_nationality': ['Character', 'Country'],
            'set_state_category': ['State'],
            'set_state_name': ['State'],
            'start_resistance': ['State'],
            'transfer_state_to': ['Country', 'State'],
            'unlock_national_focus': ['Country'],
            'activate_decision': ['Country'],
        },
        'modifiers': {
            # Already handled in not_found_fixes modifiers section above
            # No additional ambiguous fixes needed for modifiers beyond what's already covered
        },
    }
    
    for cat, fixes in ambiguous_fixes.items():
        for name, scopes in fixes.items():
            entity = get_entity(data, cat, name)
            if entity:
                old = entity.get('scopes', {}).get('usage', [])
                if set(old) != set(scopes):
                    set_scopes(entity, scopes)
                    changes.append(f"{cat}/{name}: {old} -> {scopes}")
    
    # ==============================================================
    # Report
    # ==============================================================
    if changes:
        print(f"Applied {len(changes)} scope corrections:")
        for c in changes:
            print(f"  {c}")
    else:
        print("No changes needed.")
    
    save_json(JSON_PATH, data)
    print(f"\nDone. Total changes: {len(changes)}")


if __name__ == '__main__':
    main()
