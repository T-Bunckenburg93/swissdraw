//! The components module contains all shared components for our app. Components are the building blocks of dioxus apps.
//! They can be used to defined common UI elements like buttons, forms, and modals. In this template, we define a Hero
//! component and an Echo component for fullstack apps to be used in our app.

mod hero;
pub use hero::Hero;

mod echo;
pub use echo::Echo;

mod init_db;
pub use init_db::Init_DB;

mod manually_input_teams;
pub use manually_input_teams::Input_Teams;

mod score_prior_games;
pub use score_prior_games::Score_Prior;
