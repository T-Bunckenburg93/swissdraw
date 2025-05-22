//! The views module contains the components for all Layouts and Routes for our app. Each layout and route in our [`Route`]
//! enum will render one of these components.
//!
//!
//! The [`Home`] and [`Blog`] components will be rendered when the current route is [`Route::Home`] or [`Route::Blog`] respectively.
//!
//!
//! The [`Navbar`] component will be rendered on all pages of our app since every page is under the layout. The layout defines
//! a common wrapper around all child routes.

mod home;
pub use home::Home;

mod blog;
pub use blog::Blog;

mod navbar;
pub use navbar::Navbar;

mod new_draw;
pub use new_draw::New_Draw;

mod load_draw;
pub use load_draw::Load_Draw;

mod help;
pub use help::Help;

mod score_prior_games;
pub use score_prior_games::Score_Prior_Games;

mod run_swiss_draw;
pub use run_swiss_draw::{Enter_Scores,Score_Draw};