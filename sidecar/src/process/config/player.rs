use crate::create_config;

create_config!(PlayerConfig, "player", {
    version: &'static str,
    player_types: i32,
    pt_max: i32,
    random_seed: i32,
    subs_max: i32,
    allow_mult_default_type: bool,
    catchable_area_l_stretch_max: f32,
    catchable_area_l_stretch_min: f32,
    dash_power_rate_delta_max: f32,
    dash_power_rate_delta_min: f32,
    effort_max_delta_factor: f32,
    effort_min_delta_factor: f32,
    extra_stamina_delta_max: f32,
    extra_stamina_delta_min: f32,
    foul_detect_probability_delta_factor: f32,
    inertia_moment_delta_factor: f32,
    kick_power_rate_delta_max: f32,
    kick_power_rate_delta_min: f32,
    kick_rand_delta_factor: f32,
    kickable_margin_delta_max: f32,
    kickable_margin_delta_min: f32,
    new_dash_power_rate_delta_max: f32,
    new_dash_power_rate_delta_min: f32,
    new_stamina_inc_max_delta_factor: f32,
    player_decay_delta_max: f32,
    player_decay_delta_min: f32,
    player_size_delta_factor: f32,
    player_speed_max_delta_max: f32,
    player_speed_max_delta_min: f32,
    stamina_inc_max_delta_factor: f32,
});

// impl Default for PlayerConfig {
//     fn default() -> Self {
//         Self {
//             /* player Configuration file */
//
//             // player::version
//             version: "19.0.0",
//
//             // player::player_types
//             player_types: 18,
//
//             // player::pt_max
//             pt_max: 1,
//
//             // player::random_seed
//             random_seed: - 1,
//
//             // player::subs_max
//             subs_max: 3,
//
//             // player::allow_mult_default_type
//             allow_mult_default_type: false,
//
//             // player::catchable_area_l_stretch_max
//             catchable_area_l_stretch_max: 1.3,
//
//             // player::catchable_area_l_stretch_min
//             catchable_area_l_stretch_min: 1,
//
//             // player::dash_power_rate_delta_max
//             dash_power_rate_delta_max: 0,
//
//             // player::dash_power_rate_delta_min
//             dash_power_rate_delta_min: 0,
//
//             // player::effort_max_delta_factor
//             effort_max_delta_factor: - 0.004,
//
//             // player::effort_min_delta_factor
//             effort_min_delta_factor: - 0.004,
//
//             // player::extra_stamina_delta_max
//             extra_stamina_delta_max: 50,
//
//             // player::extra_stamina_delta_min
//             extra_stamina_delta_min: 0,
//
//             // player::foul_detect_probability_delta_factor
//             foul_detect_probability_delta_factor: 0,
//
//             // player::inertia_moment_delta_factor
//             inertia_moment_delta_factor: 25,
//
//             // player::kick_power_rate_delta_max
//             kick_power_rate_delta_max: 0,
//
//             // player::kick_power_rate_delta_min
//             kick_power_rate_delta_min: 0,
//
//             // player::kick_rand_delta_factor
//             kick_rand_delta_factor: 1,
//
//             // player::kickable_margin_delta_max
//             kickable_margin_delta_max: 0.1,
//
//             // player::kickable_margin_delta_min
//             kickable_margin_delta_min: - 0.1,
//
//             // player::new_dash_power_rate_delta_max
//             new_dash_power_rate_delta_max: 0.0008,
//
//             // player::new_dash_power_rate_delta_min
//             new_dash_power_rate_delta_min: - 0.0012,
//
//             // player::new_stamina_inc_max_delta_factor
//             new_stamina_inc_max_delta_factor: - 6000,
//
//             // player::player_decay_delta_max
//             player_decay_delta_max: 0.1,
//
//             // player::player_decay_delta_min
//             player_decay_delta_min: - 0.1,
//
//             // player::player_size_delta_factor
//             player_size_delta_factor: - 100,
//
//             // player::player_speed_max_delta_max
//             player_speed_max_delta_max: 0,
//
//             // player::player_speed_max_delta_min
//             player_speed_max_delta_min: 0,
//
//             // player::stamina_inc_max_delta_factor
//             stamina_inc_max_delta_factor: 0,
//         }
//     }
// }
